//! What:
//!  Controls of the readiness withdrawal:
//!  the record a boot published goes and this route's own reader answers
//!  nothing afterwards,
//!  a boot that published none is withdrawn without a change,
//!  a file the reader refuses is removed rather than left standing,
//!  and a directory this route did not make is refused and keeps its file.
//! Why:
//!  The shutdown pass unmounts only what the record it withdrew proved,
//!  so the record the removal answers is the record the removal read,
//!  and an attach is admitted on nothing once the pass has withdrawn it
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Boot and shutdown unit").
//!  No control here mounts, unmounts or opens a device:
//!  the whole file is a folder under the test's fixture root.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const file = liveFile('withdraw'); file.publish(record());
//! const withdrawal = withdrawTheRecord(file.directory(), uid);
//! expect(withdrawal.stood).toBe('record'); expect(file.read()).toBe(null);
//! ```

use super::ReadinessFileRefusal;
use super::{
    RecordStood,
    WithdrawRefusal,
    withdraw_the_record,
};
use crate::testing::test_support::fixture;
use crate::worktree_clone_mount_protocol::clone_mount_property_names::NONCE_HEX_DIGITS;
use crate::worktree_clone_mount_protocol::clone_mount_readiness::{
    READINESS_DIRECTORY_MODE,
    ReadinessRecord,
    ReadyState,
    SweepState,
};
use crate::worktree_clone_mount_protocol::clone_mount_readiness_file::ReadinessFile;
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

/// A ready record for this boot.
fn record() -> ReadinessRecord {
    return ReadinessRecord {
        boot: String::from(BOOT),
        pool_name: String::from("zpcachyos"),
        pool_state: ReadyState::Ready,
        checkout_path: String::from("/home/user/Mono"),
        checkout_state: ReadyState::Ready,
        sweep_nonce: RequestNonce::parse("1".repeat(NONCE_HEX_DIGITS).as_str()).ok(),
        sweep_state: SweepState::Finished,
        time: String::from("2026-11-27T20:00:00-05:00"),
    };
}

/// A fresh fixture folder at the designed directory mode,
/// and the file object that expects this account.
fn live_file(name: &str) -> ReadinessFile {
    let root: PathBuf = fixture(name);
    std::fs::set_permissions(
        root.as_path(),
        std::fs::Permissions::from_mode(READINESS_DIRECTORY_MODE),
    )
    .expect("the fixture takes the directory mode");
    return ReadinessFile::at(root, this_uid());
}

/// A published record is removed, and the route's own reader answers nothing.
#[test]
fn a_published_record_is_removed_and_reads_back_as_nothing() {
    let file: ReadinessFile = live_file("readiness-withdraw-published");
    let published: ReadinessRecord = record();
    file.publish(&published).expect("the record is published");
    let withdrawal = withdraw_the_record(file.directory(), this_uid())
        .expect("the record of this boot is withdrawn");
    assert_eq!(withdrawal.stood, RecordStood::Record(Box::new(published)));
    assert_eq!(withdrawal.path, file.record());
    assert!(
        !file.record().exists(),
        "the record's name is gone after the withdrawal"
    );
    assert_eq!(
        file.read().expect("the directory still reads"),
        None,
        "this route's own reader answers nothing, which is what closes admission"
    );
}

/// A boot that published no record is withdrawn without a change.
#[test]
fn a_directory_with_no_record_is_withdrawn_and_answers_nothing() {
    let file: ReadinessFile = live_file("readiness-withdraw-absent");
    let withdrawal =
        withdraw_the_record(file.directory(), this_uid()).expect("an absent record is withdrawn");
    assert_eq!(withdrawal.stood, RecordStood::Nothing);
    assert_eq!(file.read().expect("the directory reads"), None);
}

/// A directory that is not there at all is a boot that published none,
/// and it stays absent.
#[test]
fn a_directory_that_is_not_there_is_no_record_and_no_folder_is_made() {
    let root: PathBuf = fixture("readiness-withdraw-never-made").join("never-made");
    let withdrawal = withdraw_the_record(root.as_path(), this_uid())
        .expect("a directory that is not there holds no record");
    assert_eq!(withdrawal.stood, RecordStood::Nothing);
    assert!(
        !root.exists(),
        "the withdrawal makes no folder, so nothing on this machine learns a path it did not choose"
    );
}

/// A name this route's reader refuses is removed like any other,
/// because the folder is the route's and the content is a cache.
#[test]
fn a_record_the_reader_refuses_is_removed_and_named() {
    let file: ReadinessFile = live_file("readiness-withdraw-unreadable");
    file.prepare().expect("the directory is made");
    std::fs::write(
        file.record().as_path(),
        b"v1\nkind nothing-this-route-writes\n",
    )
    .expect("the planted file is written");
    let withdrawal = withdraw_the_record(file.directory(), this_uid())
        .expect("a file the reader refuses is removed");
    match withdrawal.stood {
        RecordStood::Unreadable {
            said,
        } => {
            assert!(
                said.contains("readiness record"),
                "the reader's own sentence travels back, and this one says {said}"
            );
            assert!(
                said.contains(file.record().to_str().expect("the fixture path is text")),
                "the sentence names the file a person inspects, and this one says {said}"
            );
        }
        other => panic!("a file the reader refuses is its own answer, and this one said {other:?}"),
    }
    assert!(!file.record().exists(), "the name is gone");
}

/// A directory another account owns is refused, and its file stays.
#[test]
fn a_directory_of_another_account_is_refused_and_its_record_stays() {
    let file: ReadinessFile = live_file("readiness-withdraw-wrong-owner");
    file.publish(&record()).expect("the record is published");
    let other_owner: u32 = this_uid() + 1;
    let refusal = withdraw_the_record(file.directory(), other_owner)
        .err()
        .expect("a folder another account owns is not this route's");
    match refusal {
        WithdrawRefusal::Directory {
            refusal: inner,
        } => assert!(
            matches!(*inner, ReadinessFileRefusal::DirectoryOwner { .. }),
            "the refusal names the owner, and this one is {inner:?}"
        ),
        other => panic!("a wrong owner is the directory's own refusal, and this one is {other:?}"),
    }
    assert_eq!(
        file.read().expect("the record is read"),
        Some(record()),
        "the record of a folder this route did not prove is left exactly as it was"
    );
}

/// A directory at a mode the route does not set is refused, and its file stays.
#[test]
fn a_directory_at_another_mode_is_refused_and_its_record_stays() {
    let file: ReadinessFile = live_file("readiness-withdraw-wrong-mode");
    file.publish(&record()).expect("the record is published");
    std::fs::set_permissions(file.directory(), std::fs::Permissions::from_mode(0o777))
        .expect("the fixture takes another mode");
    let refusal = withdraw_the_record(file.directory(), this_uid())
        .err()
        .expect("a folder at an open mode is not the route's");
    assert!(
        matches!(
            refusal,
            WithdrawRefusal::Directory {
                refusal: _
            }
        ),
        "a wrong mode is the directory's own refusal, and this one is {refusal:?}"
    );
    assert_eq!(
        std::fs::read_to_string(file.record().as_path()).expect("the record's bytes are there"),
        record().text(),
        "the record of a folder at another mode is left exactly as it was"
    );
}

/// The withdrawal's refusals say what a person does next.
#[test]
fn each_refusal_names_its_path_and_the_way_forward() {
    let file: ReadinessFile = live_file("readiness-withdraw-refusal-words");
    file.publish(&record()).expect("the record is published");
    let said: String = withdraw_the_record(file.directory(), this_uid() + 1)
        .err()
        .expect("the wrong owner refuses")
        .message();
    assert!(
        said.contains("Ways forward:"),
        "a refusal says what to do next, and this one says {said}"
    );
    let still = WithdrawRefusal::StillThere {
        path: file.record(),
        said: String::from("a record"),
    };
    assert!(
        still.message().contains("admission is not closed"),
        "a record that survives says what it costs, and this one says {}",
        still.message()
    );
    let removal = WithdrawRefusal::Removal {
        path: file.record(),
        reason: String::from("a planted reason"),
    };
    assert!(
        removal.message().contains("a planted reason"),
        "a removal's own reason travels into the sentence, and this one says {}",
        removal.message()
    );
}
