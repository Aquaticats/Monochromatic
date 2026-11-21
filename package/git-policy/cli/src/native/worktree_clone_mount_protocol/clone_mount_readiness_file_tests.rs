//! What:
//!  Controls of the readiness file:
//!  the designed path,
//!  a whole publish and read,
//!  its absence,
//!  and each file or directory the route must not believe.
//! Why:
//!  The record admits an attach,
//!  so a wrong owner,
//!  mode or shape must be refused before its text is read
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "ZED reconciliation and mount safety").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const file = ReadinessFile.at(fixture('readiness'), process.getuid()); file.publish(record); file.read();
//! ```

use super::{READINESS_DIRECTORY, READINESS_DIRECTORY_MODE, READINESS_MODE, READINESS_NAME, ReadinessFile, ReadinessFileRefusal};
use crate::testing::test_support::fixture;
use crate::worktree_clone_mount_protocol::clone_mount_readiness::{ReadinessRecord, ReadyState, SweepState};
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// The boot every control belongs to.
const BOOT: &str = "0f3a5c7e-1b2d-4f60-8a9b-0c1d2e3f4a5b";

/// This account's own number.
fn this_uid() -> u32 {
    // SAFETY: `getuid` takes no argument and changes nothing.
    return unsafe { libc::getuid() };
}

/// A ready record.
fn record() -> ReadinessRecord {
    return ReadinessRecord {
        boot: String::from(BOOT),
        pool_name: String::from("zpcachyos"),
        pool_state: ReadyState::Ready,
        checkout_path: String::from("/home/user/Mono"),
        checkout_state: ReadyState::Ready,
        sweep_nonce: RequestNonce::parse("0".repeat(32).as_str()).ok(),
        sweep_state: SweepState::Finished,
        time: String::from("2026-11-21T15:00:00Z"),
    };
}

/// A fresh fixture folder with the designed directory mode,
/// and the file object that expects this account.
fn live_file(name: &str) -> ReadinessFile {
    let root: PathBuf = fixture(name);
    std::fs::set_permissions(root.as_path(), std::fs::Permissions::from_mode(READINESS_DIRECTORY_MODE)).expect("the fixture takes the directory mode");
    return ReadinessFile::at(root, this_uid());
}

/// The live place is the designed path and requires root.
#[test]
fn the_live_place_is_the_designed_one() {
    let file: ReadinessFile = ReadinessFile::live();
    assert_eq!(file.directory().to_str(), Some(READINESS_DIRECTORY));
    assert_eq!(file.record().file_name().and_then(|name| return name.to_str()), Some(READINESS_NAME));
    assert_eq!(READINESS_DIRECTORY_MODE, 0o755);
    assert_eq!(READINESS_MODE, 0o644);
}

/// A published record is `0644` and reads back whole,
/// replacing the earlier one.
#[test]
fn a_record_is_published_whole_and_reads_back() {
    let file: ReadinessFile = live_file("readiness-round-trip");
    let record: ReadinessRecord = record();
    file.publish(&record).expect("the record is published");
    let mode: u32 = std::fs::metadata(file.record().as_path()).expect("the record exists").permissions().mode() & 0o777;
    assert_eq!(mode, READINESS_MODE);
    assert!(!file.staging().exists(), "the staging sibling is renamed");
    assert_eq!(file.read().expect("the record is read"), Some(record.clone()));
    let mut next: ReadinessRecord = record;
    next.checkout_state = ReadyState::NotReady;
    file.publish(&next).expect("the next record is published");
    assert_eq!(file.read().expect("the record is read again"), Some(next));
}

/// A directory with no record reads as absent.
#[test]
fn an_unpublished_record_is_absent() {
    let file: ReadinessFile = live_file("readiness-absent");
    assert_eq!(file.read().expect("a missing record is no refusal"), None);
}

/// A wrong directory mode or owner,
/// and a wrong record mode,
/// are refused before the text is used.
#[test]
fn a_wrong_mode_or_owner_is_refused() {
    let file: ReadinessFile = live_file("readiness-directory-mode");
    file.publish(&record()).expect("the record is published");
    std::fs::set_permissions(file.directory(), std::fs::Permissions::from_mode(0o700)).expect("the directory takes another mode");
    assert!(matches!(file.read(), Err(ReadinessFileRefusal::DirectoryMode { found: 0o700, .. })));
    std::fs::set_permissions(file.directory(), std::fs::Permissions::from_mode(READINESS_DIRECTORY_MODE)).expect("the directory takes its mode back");
    std::fs::set_permissions(file.record().as_path(), std::fs::Permissions::from_mode(0o600)).expect("the record takes another mode");
    assert!(matches!(file.read(), Err(ReadinessFileRefusal::FileMode { found: 0o600 })));
    let wrong_owner: ReadinessFile = ReadinessFile::at(file.directory().to_path_buf(), this_uid() + 1);
    assert!(matches!(wrong_owner.read(), Err(ReadinessFileRefusal::DirectoryOwner { .. })));
}

/// A link named after the record is not a record,
/// and a foreign file or text is refused.
#[test]
fn a_link_or_foreign_file_is_refused() {
    let file: ReadinessFile = live_file("readiness-shapes");
    let elsewhere: PathBuf = file.directory().join("elsewhere");
    std::fs::write(elsewhere.as_path(), record().text()).expect("the other file is written");
    std::fs::set_permissions(elsewhere.as_path(), std::fs::Permissions::from_mode(READINESS_MODE)).expect("the other file takes the mode");
    std::os::unix::fs::symlink(elsewhere.as_path(), file.record()).expect("the link is made");
    assert!(matches!(file.read(), Err(ReadinessFileRefusal::File { .. })));
    std::fs::remove_file(file.record()).expect("the link is removed");
    std::fs::write(file.record().as_path(), "v1\nkind something-else\n").expect("the foreign file is written");
    assert!(matches!(file.read(), Err(ReadinessFileRefusal::NotAReadiness { .. })));
    std::fs::write(file.record().as_path(), [0xff_u8]).expect("the binary file is written");
    assert!(matches!(file.read(), Err(ReadinessFileRefusal::NotText)));
}

/// Every refusal says nothing was attached on the record.
#[test]
fn every_refusal_says_nothing_was_attached() {
    let file: ReadinessFile = ReadinessFile::at(PathBuf::from("/nonexistent/readiness"), 0);
    let refusal: ReadinessFileRefusal = file.read().expect_err("a missing directory is refused");
    assert!(refusal.message().contains("Nothing was attached"), "{}", refusal.message());
}
