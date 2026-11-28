//! What:
//!  The controls of the readiness record:
//!  a record the handler's own writer published,
//!  a folder with no record,
//!  and a record this build cannot parse.
//! Why:
//!  A caller waits for this record rather than for an installed executable,
//!  so a record's boot and two states must arrive whole,
//!  and a folder nobody read must answer unknown rather than a record that
//!  admits nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theReadinessFacts(readinessFile)).toMatchObject({ kind: 'read', boot: LIVE_BOOT });
//! ```

/// The read a control judges.
use super::the_readiness_facts;
/// What the latest record answered.
use crate::doctor_zfs_route::handler_checks_readiness::ReadinessFacts;
/// The names the design's layout fixes.
use crate::doctor_zfs_route::route_test_facts::LIVE_BOOT;
/// One fixture folder of this call's own.
use crate::testing::test_support::fixture;
/// The account this process runs as.
use crate::worktree_clone_add_route::add_grants::this_account;
/// The handler's reader of the record,
/// and the record itself with its states.
use crate::worktree_clone_mount_protocol::{
    ReadinessFile,
    ReadinessRecord,
    ReadyState,
    SweepState,
};
/// The owned paths these controls use.
use std::path::{
    Path,
    PathBuf,
};

/// One record at this boot's state.
fn the_record() -> ReadinessRecord {
    return ReadinessRecord {
        boot: String::from(LIVE_BOOT),
        pool_name: String::from("zpcachyos"),
        pool_state: ReadyState::Ready,
        checkout_path: String::from("/home/user/Monochromatic"),
        checkout_state: ReadyState::Ready,
        sweep_nonce: None,
        sweep_state: SweepState::Finished,
        time: String::from("2026-11-27T00:00:00+00:00"),
    };
}

/// One reader at a fixture folder the control made.
fn the_reader(name: &str) -> ReadinessFile {
    let root: PathBuf = fixture(name);
    let folder: PathBuf = root.join("run/git-worktree-clone-mount-readiness");
    std::fs::create_dir_all(folder.as_path()).expect("the readiness folder");
    std::fs::set_permissions(
        folder.as_path(),
        std::os::unix::fs::PermissionsExt::from_mode(0o755),
    )
    .expect("the readiness folder's mode");
    return ReadinessFile::at(folder, this_account());
}

/// What:
///  A record the handler's writer published answers its boot,
///  its two states and the word its sweep state is written as.
/// Why:
///  The readiness check compares this boot with the record's and both states
///  with ready,
///  so the three fields are what the report reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(theReadinessFacts(reader)).toMatchObject({ kind: 'read', boot: LIVE_BOOT, pool: 'ready', checkout: 'ready' });
/// ```
#[test]
fn a_published_record_answers_its_boot_and_its_states() {
    let reader: ReadinessFile = the_reader("readiness-published");
    reader
        .publish(&the_record())
        .expect("the record the handler publishes");
    let facts: ReadinessFacts = the_readiness_facts(&reader);
    let ReadinessFacts::Read {
        boot,
        pool,
        checkout,
        sweep,
    } = facts
    else {
        panic!("a published record is read: {facts:?}");
    };
    assert_eq!(boot, LIVE_BOOT);
    assert_eq!(pool, ReadyState::Ready);
    assert_eq!(checkout, ReadyState::Ready);
    assert_eq!(sweep, SweepState::Finished.word());
}

/// What:
///  A folder with no record answers an absence naming the file,
///  and a record no reader accepts answers unknown naming the folder.
/// Why:
///  A missing record sends a person to the handler's log and an unreadable one
///  to the folder's owner or mode,
///  so the two must not answer one sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(theReadinessFacts(reader)).toEqual({ kind: 'absent', why });
/// ```
#[test]
fn no_record_is_an_absence_and_one_nobody_reads_is_unknown() {
    let reader: ReadinessFile = the_reader("readiness-absent");
    let facts: ReadinessFacts = the_readiness_facts(&reader);
    let ReadinessFacts::Absent {
        why,
    } = facts
    else {
        panic!("no record is an absence: {facts:?}");
    };
    assert!(why.contains("readiness"), "{why}");
    let broken: ReadinessFile = the_reader("readiness-broken");
    write_a_record(broken.record().as_path(), "not a readiness record\n");
    let unreadable: ReadinessFacts = the_readiness_facts(&broken);
    assert!(
        matches!(unreadable, ReadinessFacts::Unreadable { .. }),
        "a record nobody reads is unknown: {unreadable:?}"
    );
}

/// One record file written by hand,
/// which is a control's own text and no writer's.
fn write_a_record(
    path: &Path,
    text: &str,
) {
    std::fs::write(path, text.as_bytes()).expect("a record file");
    std::fs::set_permissions(path, std::os::unix::fs::PermissionsExt::from_mode(0o644))
        .expect("the record's mode");
}
