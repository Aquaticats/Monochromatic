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

use super::{
    READINESS_DIRECTORY, READINESS_DIRECTORY_MODE, READINESS_MODE, READINESS_NAME, ReadinessFile, ReadinessFileRefusal, readiness_staging_name,
};
use crate::testing::test_support::fixture;
use crate::worktree_clone_mount_protocol::clone_mount_readiness::{ReadinessRecord, ReadyState, SweepState};
use crate::worktree_clone_mount_protocol::clone_mount_property_names::NONCE_HEX_DIGITS;
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
        sweep_nonce: RequestNonce::parse("0".repeat(NONCE_HEX_DIGITS).as_str()).ok(),
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
    let file: ReadinessFile = live_file("readiness-refusal-words");
    std::fs::set_permissions(file.directory(), std::fs::Permissions::from_mode(0o700)).expect("the directory takes another mode");
    let refusal: ReadinessFileRefusal = file.read().expect_err("another mode is refused");
    assert!(refusal.message().contains("Nothing was attached"), "{}", refusal.message());
}

/// What:
///  A first publish in an empty parent makes the directory at its designed mode
///  and files the record,
///  which is what a fresh boot needs.
/// Why:
///  Nothing else creates the readiness directory,
///  so a publish that refused without it could not work after a boot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const file = ReadinessFile.at(empty.join('never-made'), uid); file.publish(record); expect(file.read()).toEqual(record);
/// ```
#[test]
fn a_first_publish_makes_its_directory() {
    let parent: PathBuf = fixture("readiness-fresh-boot");
    let directory: PathBuf = parent.join("never-made");
    let file: ReadinessFile = ReadinessFile::at(directory.clone(), this_uid());
    let record: ReadinessRecord = record();
    file.publish(&record).expect("the first publish makes its directory");
    let mode: u32 = std::fs::metadata(directory.as_path()).expect("the directory exists").permissions().mode() & 0o777;
    assert_eq!(mode, READINESS_DIRECTORY_MODE);
    assert_eq!(file.read().expect("the record is read"), Some(record));
}

/// What:
///  A boot that has published no readiness has no directory either,
///  and its read answers "no record" rather than a refusal.
/// Why:
///   The directory is made by the first publish,
///  so a reader that refused an absent directory would refuse every attach
///  between the boot and the handler's first publish with the wrong reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(ReadinessFile.at(neverMade, uid).read()).toBeNull();
/// ```
#[test]
fn an_unpublished_boot_has_no_directory_and_no_refusal() {
    let file: ReadinessFile = ReadinessFile::at(fixture("readiness-no-directory").join("never-made"), this_uid());
    assert_eq!(file.read().expect("an absent directory is no record yet"), None);
}

/// What:
///  Two publishes of one record in one process, at the same time,
///  leave one whole record and no staging sibling.
/// Why:
///   The record has several writers once the boot unit and the sweeps land,
///  and a staging name shared by two publishes would let the second remove the
///  first one's live file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await Promise.all([file.publish(a), file.publish(b)]); expect(file.staging()).not.toExist();
/// ```
#[test]
fn two_publishes_at_once_leave_one_whole_record() {
    let file: ReadinessFile = live_file("readiness-two-writers");
    assert_ne!(readiness_staging_name(), readiness_staging_name(), "two attempts never share a staging name");
    let mut first: ReadinessRecord = record();
    let mut second: ReadinessRecord = record();
    first.time = String::from("2026-11-21T15:00:01Z");
    second.checkout_state = ReadyState::NotReady;
    second.time = String::from("2026-11-21T15:00:02Z");
    let gate: std::sync::Barrier = std::sync::Barrier::new(2);
    std::thread::scope(|scope| {
        for record in [&first, &second] {
            let writer: ReadinessFile = file.clone();
            let publishing: ReadinessRecord = record.clone();
            let barrier: &std::sync::Barrier = &gate;
            scope.spawn(move || {
                barrier.wait();
                writer.publish(&publishing).expect("a writer publishes its record");
            });
        }
    });
    let found: Option<ReadinessRecord> = file.read().expect("the record is read");
    assert!(found.as_ref() == Some(&first) || found.as_ref() == Some(&second), "one whole record is the answer: {found:?}");
    let left: Vec<PathBuf> = std::fs::read_dir(file.directory())
        .expect("the directory lists")
        .flatten()
        .map(|entry| return entry.path())
        .filter(|path| return path.to_string_lossy().contains(".staging."))
        .collect();
    assert!(left.is_empty(), "no staging sibling survives: {left:?}");
}

/// What:
///  A readiness record this route's own reader would refuse is never written,
///  and a leftover staging sibling of this process does not block the retry.
/// Why:
///   A record nobody can read would make this boot look unready for reasons the
///  handler never saw,
///  and a failed publish must not block the same process's next one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(() => file.publish(hostile)).toThrow(); expect(publishAgain()).toBeUndefined();
/// ```
#[test]
fn a_record_no_reader_accepts_is_not_written_and_a_leftover_does_not_block() {
    let file: ReadinessFile = live_file("readiness-hostile");
    let mut hostile: ReadinessRecord = record();
    hostile.pool_name = String::from("pool name with spaces");
    let refusal: ReadinessFileRefusal = file.publish(&hostile).expect_err("a pool name with a space is no record");
    assert!(matches!(refusal, ReadinessFileRefusal::NotAReadiness { .. }), "{refusal:?}");
    assert!(!file.record().exists(), "nothing was filed");
    std::fs::write(file.staging(), record().text()).expect("this process's earlier sibling is planted");
    file.publish(&record()).expect("a sibling of this process's own name blocks nothing");
    assert!(!file.staging().exists(), "the sibling is renamed over the record");
}
