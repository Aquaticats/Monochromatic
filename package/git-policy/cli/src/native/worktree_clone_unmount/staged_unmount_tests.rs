//! What:
//!  Controls for the later sweep's reading of a record.
//! Why:
//!  A sweep is a non-binding pass:
//!  an absent,
//!  void or unrecorded record unmounts nothing and reports which it found,
//!  and an empty folder on the cache's own mount is empty rather than a mount
//!  to take down.
//!  The unmount itself needs a mount and is proved in the guest.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(finishAStagedUnmount(sweep).word).toBe('empty');
//! ```

use super::{finish_a_staged_unmount, StagedSweep, SweepOutcome};
use crate::staged_folder_identity::StagedFolderIdentity;
use crate::staged_folder_record::{write_record, StagedFolderRecord};
use crate::staging_calls::StagingCalls;
use crate::staging_calls_control::RecordingCalls;
use crate::staging_folder_name::staging_folder_name;
use crate::user_cache_pin::UserCachePin;
use crate::worktree_clone_mount_protocol::DatasetGuid;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

/// The GUID the fixtures use.
fn guid() -> DatasetGuid {
    return DatasetGuid::parse("808").expect("the fixture GUID parses");
}

/// A cache fixture with a staging folder and one record of this boot.
fn fixture(name: &str) -> (std::path::PathBuf, std::fs::File, UserCachePin) {
    let root: PathBuf = crate::testing::test_support::fixture_root().join(name);
    std::fs::create_dir_all(root.as_path()).expect("the fixture cache is made");
    let folder_path: PathBuf = root.join(staging_folder_name(guid()));
    std::fs::create_dir_all(folder_path.as_path()).expect("the fixture folder is made");
    let cache: std::fs::File = std::fs::File::open(root.as_path()).expect("the fixture cache opens");
    let folder: std::fs::File = std::fs::File::open(folder_path.as_path()).expect("the fixture folder opens");
    let record: StagedFolderRecord = StagedFolderRecord {
        component: staging_folder_name(guid()),
        dataset: String::from("pool/clones/one"),
        guid: guid(),
        identity: StagedFolderIdentity::of(&folder).expect("the fixture identity is read"),
        handle_type: 1,
        handle: vec![1, 2, 3],
        nonce: String::from("0123456789abcdef0123456789abcdef"),
        namespace_device: 7,
        namespace_inode: 9,
        boot: String::from("this-boot"),
    };
    write_record(&cache, &record).expect("the record is written");
    let pin: UserCachePin = UserCachePin { source: String::from("pool/cache"), root_inode: cache.metadata().expect("the cache's metadata is read").ino(), device: 1 };
    return (root, cache, pin);
}

/// An absent record is reported and nothing is unmounted.
#[test]
fn an_absent_record_is_reported() {
    let root: PathBuf = crate::testing::test_support::fixture_root().join("sweep-absent");
    std::fs::create_dir_all(root.as_path()).expect("the fixture cache is made");
    let cache: std::fs::File = std::fs::File::open(root.as_path()).expect("the fixture cache opens");
    let pin: UserCachePin = UserCachePin { source: String::from("pool/cache"), root_inode: cache.metadata().expect("the cache's metadata is read").ino(), device: 1 };
    let calls = RecordingCalls::default();
    let sweep = StagedSweep { cache: &cache, cache_path: root.as_path(), guid: guid(), boot: "this-boot", pin: &pin, calls: &calls };
    let outcome: SweepOutcome = finish_a_staged_unmount(&sweep);
    assert_eq!(outcome.word(), "not-current");
    assert!(!calls.ran(crate::staging_calls::StagingCall::Unmount));
}

/// A record from another boot is void and its handle is not opened.
#[test]
fn another_boots_record_is_void() {
    let (root, cache, pin) = fixture("sweep-other-boot");
    let mut record: StagedFolderRecord = crate::staged_folder_record::read_record_text(&cache, guid()).expect("the record is read").map(|text| return crate::staged_folder_record::record_from_text(text.as_str(), guid()).expect("the record parses")).expect("the record is there");
    record.boot = String::from("an-older-boot");
    write_record(&cache, &record).expect("the older record is written");
    let calls = RecordingCalls::default();
    let sweep = StagedSweep { cache: &cache, cache_path: root.as_path(), guid: guid(), boot: "this-boot", pin: &pin, calls: &calls };
    let outcome: SweepOutcome = finish_a_staged_unmount(&sweep);
    assert_eq!(outcome.word(), "not-current");
    assert!(!calls.ran(crate::staging_calls::StagingCall::OpenHandle));
}

/// A current record is read,
/// and the pin is the next check before any unmount.
#[test]
fn a_current_record_reaches_the_pin_before_any_unmount() {
    let (root, cache, pin) = fixture("sweep-empty");
    let folder_path: PathBuf = root.join(staging_folder_name(guid()));
    let folder: std::fs::File = std::fs::File::open(folder_path.as_path()).expect("the fixture folder opens");
    let calls = RecordingCalls::default();
    *calls.open_handle.borrow_mut() = Some(Ok(folder));
    calls.mount_ids.borrow_mut().extend([7, 7, 7]);
    let sweep = StagedSweep { cache: &cache, cache_path: root.as_path(), guid: guid(), boot: "this-boot", pin: &pin, calls: &calls };
    let outcome: SweepOutcome = finish_a_staged_unmount(&sweep);
    assert_eq!(outcome.word(), "empty", "{outcome:?}");
    assert!(!calls.ran(crate::staging_calls::StagingCall::Unmount));
}

/// A current record whose folder path a mount row names is not empty,
/// even though the handle reopens the folder on the cache mount.
#[test]
fn a_current_record_with_a_row_is_not_empty() {
    let (root, cache, pin) = fixture("sweep-row");
    let folder_path: PathBuf = root.join(staging_folder_name(guid()));
    let folder: std::fs::File = std::fs::File::open(folder_path.as_path()).expect("the fixture folder opens");
    let calls = RecordingCalls::default();
    *calls.open_handle.borrow_mut() = Some(Ok(folder));
    calls.mount_ids.borrow_mut().extend([7, 7]);
    *calls.table.borrow_mut() = vec![crate::fs_mount_options::MountFacts {
        mount_id: 51,
        parent_id: 7,
        root: PathBuf::from("/"),
        optional_fields: Vec::new(),
        mount_point: folder_path.clone(),
        filesystem_type: String::from("zfs"),
        source: String::from("tmpfs"),
        mount_options: Vec::new(),
        super_options: Vec::new(),
    }];
    let sweep = StagedSweep { cache: &cache, cache_path: root.as_path(), guid: guid(), boot: "this-boot", pin: &pin, calls: &calls };
    let outcome: SweepOutcome = finish_a_staged_unmount(&sweep);
    assert_ne!(outcome.word(), "empty", "a row at the folder path is not an empty folder: {outcome:?}");
}

/// What:
///  A holder inside the staged mount makes the unmount answer `EBUSY`,
///  and the same call succeeds once the holder ends.
/// Why:
///  The busy answer must be the kernel's,
/// not a timed guess,
///  so the test starts its own holder and ends it between the two calls.
///  The mounts live in the user and mount namespace the control is re-run in,
///  so the host's table is never touched.
#[test]
fn a_holder_makes_the_staged_unmount_busy_then_it_succeeds() {
    let test: &str = "worktree_clone_unmount::staged_unmount::tests::a_holder_makes_the_staged_unmount_busy_then_it_succeeds";
    if !in_a_mount_namespace("STAGED_UNMOUNT_BUSY_INSIDE", test) {
        return;
    }
    let root: PathBuf = crate::testing::test_support::fixture_root().join("staged-unmount-busy");
    let cache_path: PathBuf = root.join("cache");
    let staging: PathBuf = cache_path.join(staging_folder_name(guid()));
    let clone_path: PathBuf = root.join("clone");
    std::fs::create_dir_all(staging.as_path()).expect("the staging folder is made");
    std::fs::create_dir_all(clone_path.as_path()).expect("the clone folder is made");
    let cache: std::fs::File = std::fs::File::open(cache_path.as_path()).expect("the cache opens");
    let folder: std::fs::File = std::fs::File::open(staging.as_path()).expect("the staging folder opens");
    let bind: std::fs::File = crate::staging_calls::LiveStagingCalls.bind_the_folder(&folder, &cache, std::ffi::OsStr::new(staging_folder_name(guid()).as_str())).expect("the folder binds");
    crate::staging_calls::LiveStagingCalls.make_private(&bind).expect("the bind is private");
    let clone: std::fs::File = mount_a_tmpfs(clone_path.as_path());
    crate::staging_calls::LiveStagingCalls.move_clone(&clone, &bind).expect("the tmpfs moves onto the bind");
    // The descriptors on the moved mount and the bind would pin them, which is the EBUSY the design says to release first.
    drop(clone);
    drop(bind);
    // `folder` was opened before the bind and is therefore on the cache mount, which is the held descriptor the design
    // gives the unmount; a fresh open of the path would land on the moved mount and pin it.
    let mut holder: std::process::Child = std::process::Command::new("sh").args(["-c", format!("cd {} && exec sleep 60", staging.display()).as_str()]).spawn().expect("the holder starts");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while std::time::Instant::now() < deadline {
        if std::fs::read_link(format!("/proc/{}/cwd", holder.id())).ok() == std::fs::canonicalize(staging.as_path()).ok() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let busy = super::unmount_the_staged_mount(&super::HeldUnmount { cache: &cache, folder: &folder, name: staging_folder_name(guid()).as_str(), clone_source: "tmpfs", calls: &crate::staging_calls::LiveStagingCalls });
    assert!(matches!(busy, super::HeldOutcome::Retained { ref step, errno: libc::EBUSY, .. } if step == "unmount"), "the holder forces EBUSY: {busy:?}");
    let _ = holder.kill();
    let _ = holder.wait();
    let free = super::unmount_the_staged_mount(&super::HeldUnmount { cache: &cache, folder: &folder, name: staging_folder_name(guid()).as_str(), clone_source: "tmpfs", calls: &crate::staging_calls::LiveStagingCalls });
    assert!(matches!(free, super::HeldOutcome::Unmounted { .. }), "the unmount succeeds after the holder ends: {free:?}");
}

/// Mount a fresh tmpfs at one fixture folder and answer a descriptor on it.
fn mount_a_tmpfs(target: &std::path::Path) -> std::fs::File {
    use std::os::unix::ffi::OsStrExt;
    let at = std::ffi::CString::new(target.as_os_str().as_bytes()).expect("a path holds no NUL");
    let kind = std::ffi::CString::new("tmpfs").expect("no NUL in a type");
    let data = std::ffi::CString::new("size=64k,nr_inodes=16,mode=0700").expect("no NUL in an option");
    // SAFETY: every pointer is this call's own and outlives it.
    let mounted: libc::c_int = unsafe { libc::mount(kind.as_ptr(), at.as_ptr(), kind.as_ptr(), libc::MS_NOSUID | libc::MS_NODEV, data.as_ptr().cast::<libc::c_void>()) };
    assert_eq!(mounted, 0, "the fixture tmpfs mounts: {}", std::io::Error::last_os_error());
    return std::fs::File::open(target).expect("the mounted folder opens");
}

/// Whether this run is the inner half of a private user and mount namespace,
/// and the re-execution that makes it one.
/// Why:
///  A bind and a mount need both namespaces,
///  and the file-ownership helper unshares only the user one.
fn in_a_mount_namespace(variable: &str, test: &str) -> bool {
    if std::env::var_os(variable).is_some() {
        return true;
    }
    // The inner half is this test binary in a mount namespace of its own, and its pass is this control's assertion.
    let environment: [(&str, &std::ffi::OsStr); 1] = [(variable, std::ffi::OsStr::new("1"))];
    crate::child_test_process::passed(crate::child_test_process::ChildTest {
        exact_test: test,
        environment: environment.as_slice(),
        running_program: Some(crate::child_test_process::ProgramRunningTheTest {
            program: "unshare",
            arguments_before_the_test_binary: &["--user", "--map-root-user", "--mount", "--propagation", "private"],
        }),
        ..crate::child_test_process::ChildTest::default()
    });
    return false;
}

/// A current record with no row at the folder is finished by the next run.
#[test]
fn a_finished_state_is_finished_by_the_next_run() {
    let (root, cache, _pin) = fixture("recover-finished");
    let facts = crate::user_cache_pin::cache_mount_facts(&cache).expect("the cache row reads");
    let pin = UserCachePin { source: facts.source.clone(), root_inode: cache.metadata().expect("the cache metadata reads").ino(), device: 1 };
    let calls = RecordingCalls::default();
    let held = std::fs::File::open(root.join(staging_folder_name(guid())).as_path()).expect("the staged folder opens");
    *calls.open_handle.borrow_mut() = Some(Ok(held));
    *calls.facts.borrow_mut() = Some(crate::user_cache_pin::cache_mount_facts(&cache).expect("the cache row reads"));
    calls.mount_ids.borrow_mut().extend([7]);
    let sweep = StagedSweep { cache: &cache, cache_path: root.as_path(), guid: guid(), boot: "this-boot", pin: &pin, calls: &calls };
    let slot: PathBuf = root.join("slot");
    assert_eq!(crate::staged_unmount::recover_a_staged_unmount(&sweep, slot.as_path(), 41), crate::staged_unmount::RecoveryOutcome::Finished { mount_id: 41 });
}

/// A foreign mount at the folder refuses rather than being taken for the clone.
#[test]
fn a_foreign_mount_refuses_recovery() {
    let (root, cache, _pin) = fixture("recover-foreign");
    let facts = crate::user_cache_pin::cache_mount_facts(&cache).expect("the cache row reads");
    let pin = UserCachePin { source: facts.source.clone(), root_inode: cache.metadata().expect("the cache metadata reads").ino(), device: 1 };
    let calls = RecordingCalls::default();
    let held = std::fs::File::open(root.join(staging_folder_name(guid())).as_path()).expect("the staged folder opens");
    *calls.open_handle.borrow_mut() = Some(Ok(held));
    *calls.facts.borrow_mut() = Some(crate::user_cache_pin::cache_mount_facts(&cache).expect("the cache row reads"));
    calls.mount_ids.borrow_mut().extend([7]);
    let folder_path: PathBuf = root.join(staging_folder_name(guid()));
    *calls.table.borrow_mut() = vec![crate::fs_mount_options::MountFacts {
        mount_id: 51,
        parent_id: 7,
        root: PathBuf::from("/"),
        optional_fields: Vec::new(),
        mount_point: folder_path,
        filesystem_type: String::from("tmpfs"),
        source: String::from("foreign"),
        mount_options: Vec::new(),
        super_options: Vec::new(),
    }];
    let sweep = StagedSweep { cache: &cache, cache_path: root.as_path(), guid: guid(), boot: "this-boot", pin: &pin, calls: &calls };
    let outcome = crate::staged_unmount::recover_a_staged_unmount(&sweep, root.join("slot").as_path(), 41);
    assert!(matches!(outcome, crate::staged_unmount::RecoveryOutcome::Refused { ref step, .. } if step == "pre-call-check"), "{outcome:?}");
}

/// A bind left over an empty folder with the clone gone is unbound and finished.
#[test]
fn a_leftover_bind_is_unbound_and_finished() {
    let (root, cache, _pin) = fixture("recover-bind");
    let facts = crate::user_cache_pin::cache_mount_facts(&cache).expect("the cache row reads");
    let pin = UserCachePin { source: facts.source.clone(), root_inode: cache.metadata().expect("the cache metadata reads").ino(), device: 1 };
    let calls = RecordingCalls::default();
    let held = std::fs::File::open(root.join(staging_folder_name(guid())).as_path()).expect("the staged folder opens");
    *calls.open_handle.borrow_mut() = Some(Ok(held));
    *calls.facts.borrow_mut() = Some(crate::user_cache_pin::cache_mount_facts(&cache).expect("the cache row reads"));
    calls.mount_ids.borrow_mut().extend([7]);
    let source = crate::user_cache_pin::cache_mount_facts(&cache).expect("the cache's row reads").source;
    let folder_path: PathBuf = root.join(staging_folder_name(guid()));
    *calls.table.borrow_mut() = vec![crate::fs_mount_options::MountFacts {
        mount_id: 52,
        parent_id: 7,
        root: folder_path.clone(),
        optional_fields: Vec::new(),
        mount_point: folder_path,
        filesystem_type: source.clone(),
        source,
        mount_options: Vec::new(),
        super_options: Vec::new(),
    }];
    let sweep = StagedSweep { cache: &cache, cache_path: root.as_path(), guid: guid(), boot: "this-boot", pin: &pin, calls: &calls };
    let outcome = crate::staged_unmount::recover_a_staged_unmount(&sweep, root.join("slot").as_path(), 41);
    assert_eq!(outcome, crate::staged_unmount::RecoveryOutcome::Finished { mount_id: 41 }, "{outcome:?}");
    assert!(calls.ran(crate::staging_calls::StagingCall::Unmount));
}
