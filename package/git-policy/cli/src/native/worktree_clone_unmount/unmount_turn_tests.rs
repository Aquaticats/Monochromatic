//! What:
//!  Controls for every identity comparison the unmount makes before a move.
//! Why:
//!  Each arm changes one expected field and asserts the refusal names it and
//!  that no bind or move ran;
//!  the passing arm reaches the pin,
//!  which is the next refusal this host can give without a mount.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = unmountTheClone({ ...request, expected: { ...expected, mountId: 42 } });
//! expect(outcome.step).toBe('mount-id'); expect(calls.ran('bind')).toBe(false);
//! ```

use crate::staged_folder_identity::StagedFolderIdentity;
use crate::unmount_checks::ExpectedMount;
use crate::unmount_turn::{unmount_the_clone, UnmountOutcome, UnmountRequest};
use crate::slot_walk_calls::{DatasetGuids, SlotWalk};
use crate::staging_calls::{StagingCall, StagingCallRefusal};
use crate::staging_calls_control::RecordingCalls;
use crate::staging_folder_name::staging_folder_name;
use crate::user_cache_pin::UserCachePin;
use crate::worktree_clone_mount_protocol::DatasetGuid;
use std::path::{Path, PathBuf};

/// The GUID every arm uses.
fn guid() -> DatasetGuid {
    return DatasetGuid::parse("77").expect("the fixture GUID parses");
}

/// The walk a control drives:
/// one fixture folder in place of the slot's mount.
struct FakeWalk {
    slot: PathBuf,
    again: u64,
}

impl SlotWalk for FakeWalk {
    fn open_the_slot(&self, _checkout: &Path, _slot: &str) -> Result<std::fs::File, StagingCallRefusal> {
        return std::fs::File::open(self.slot.as_path()).map_err(|error| return StagingCallRefusal { call: StagingCall::OpenSlot, errno: error.raw_os_error().unwrap_or(0), detail: error.to_string() });
    }

    fn open_the_slot_uncrossed(&self, _checkout: &Path, _slot: &str) -> Result<std::fs::File, StagingCallRefusal> {
        return self.open_the_slot(_checkout, _slot);
    }

    fn look_again(&self, _checkout: &Path, _slot: &str) -> Result<u64, StagingCallRefusal> {
        return Ok(self.again);
    }
}

/// The GUID read a control drives.
struct FakeGuids {
    guid: DatasetGuid,
}

impl DatasetGuids for FakeGuids {
    fn live_guid(&self, _dataset: &str) -> Result<DatasetGuid, String> {
        return Ok(self.guid);
    }
}

/// Every fixture one arm needs.
struct Fixture {
    checkout: PathBuf,
    cache_path: PathBuf,
    cache: std::fs::File,
    mount: u64,
    expected: ExpectedMount,
    slot_identity: StagedFolderIdentity,
    pin: UserCachePin,
    calls: RecordingCalls,
}

/// Make the checkout,
/// the slot,
/// the cache and its staging folder.
fn fixture(name: &str) -> Fixture {
    let checkout: PathBuf = crate::testing::test_support::fixture_root().join(name);
    let slot: PathBuf = checkout.join(".claude/worktrees/one");
    std::fs::create_dir_all(slot.as_path()).expect("the fixture slot is made");
    let cache_path: PathBuf = checkout.join("home/.cache");
    std::fs::create_dir_all(cache_path.as_path()).expect("the fixture cache is made");
    let component: String = staging_folder_name(guid());
    let staging: PathBuf = cache_path.join(component.as_str());
    std::fs::create_dir_all(staging.as_path()).expect("the fixture staging folder is made");
    std::fs::set_permissions(staging.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o700)).expect("the fixture staging folder holds 0700");
    let cache: std::fs::File = std::fs::File::open(cache_path.as_path()).expect("the fixture cache opens");
    let facts: std::fs::Metadata = std::fs::metadata(slot.as_path()).expect("the slot's metadata is read");
    use std::os::unix::fs::MetadataExt;
    let slot_identity: StagedFolderIdentity = StagedFolderIdentity::of(&std::fs::File::open(slot.as_path()).expect("the slot opens")).expect("the slot's identity is read");
    let pin: UserCachePin = UserCachePin { source: String::from("pool/cache"), root_inode: cache.metadata().expect("the cache's metadata is read").ino(), device: 1 };
    return Fixture {
        checkout,
        cache_path,
        cache,
        mount: 41,
        expected: ExpectedMount {
            mount_id: 41,
            parent_mount_id: 40,
            device: facts.dev(),
            inode: facts.ino(),
            canonical_slot: slot.clone(),
            source: String::from("pool/clones/one"),
            guid: guid(),
        },
        slot_identity,
        pin,
        calls: RecordingCalls { observed: slot.clone(), mount_id: 41, ..RecordingCalls::default() },
    };
}

/// The row every arm starts from.
fn row(expected: &ExpectedMount) -> crate::fs_mount_options::MountFacts {
    return crate::fs_mount_options::MountFacts {
        mount_id: expected.mount_id,
        parent_id: expected.parent_mount_id,
        root: PathBuf::from("/"),
        optional_fields: Vec::new(),
        mount_point: expected.canonical_slot.clone(),
        filesystem_type: String::from("zfs"),
        source: expected.source.clone(),
        mount_options: Vec::new(),
        super_options: Vec::new(),
    };
}

/// Run one arm with the fixture's facts and a mutation of the expectation.
fn run(fixture: &Fixture, change: impl FnOnce(&mut ExpectedMount, &mut RecordingCalls)) -> UnmountOutcome {
    let mut expected: ExpectedMount = fixture.expected.clone();
    let mut calls = RecordingCalls { observed: fixture.calls.observed.clone(), mount_id: fixture.mount, table: std::cell::RefCell::new(Vec::new()), ..RecordingCalls::default() };
    change(&mut expected, &mut calls);
    *calls.facts.borrow_mut() = Some(row(&fixture.expected));
    *calls.table.borrow_mut() = vec![row(&fixture.expected)];
    let walk = FakeWalk { slot: fixture.expected.canonical_slot.clone(), again: fixture.expected.mount_id };
    let guids = FakeGuids { guid: guid() };
    let request = UnmountRequest {
        checkout: fixture.checkout.as_path(),
        slot: "one",
        nonce: "0123456789abcdef0123456789abcdef",
        expected: &expected,
        slot_identity: &fixture.slot_identity,
        pin: &fixture.pin,
        cache: &fixture.cache,
        cache_path: fixture.cache_path.as_path(),
        boot: "this-boot",
        calls: &calls,
        walk: &walk,
        guids: &guids,
    };
    let outcome: UnmountOutcome = unmount_the_clone(&request);
    assert!(!calls.ran(StagingCall::Bind), "an arm that refuses before the bind makes no bind call");
    assert!(!calls.ran(StagingCall::Move), "an arm that refuses before the bind makes no move call");
    return outcome;
}

/// The fixture's own expectation passes every check.
#[test]
fn the_fixtures_expectation_passes_the_checks() {
    let fixture = fixture("unmount-turn-passes");
    let outcome: UnmountOutcome = run(&fixture, |_expected, _calls| {});
    assert_eq!(outcome.word(), "refused");
    match outcome {
        UnmountOutcome::Refused { step, .. } => assert_eq!(step, "user-cache-pin"),
        other => panic!("the passing arm reached the pin, and it answered {other:?}"),
    }
}

/// A mismatched mount id refuses before any move.
#[test]
fn a_mismatched_mount_id_refuses() {
    let fixture = fixture("unmount-turn-mount-id");
    let outcome: UnmountOutcome = run(&fixture, |expected, _calls| expected.mount_id = 42);
    assert!(matches!(outcome, UnmountOutcome::Refused { ref step, .. } if step == "mount-id"), "{outcome:?}");
}

/// A mismatched device refuses before any move.
#[test]
fn a_mismatched_device_refuses() {
    let fixture = fixture("unmount-turn-device");
    let outcome: UnmountOutcome = run(&fixture, |expected, _calls| expected.device += 1);
    assert!(matches!(outcome, UnmountOutcome::Refused { ref step, .. } if step == "device"), "{outcome:?}");
}

/// A mismatched inode refuses before any move.
#[test]
fn a_mismatched_inode_refuses() {
    let fixture = fixture("unmount-turn-inode");
    let outcome: UnmountOutcome = run(&fixture, |expected, _calls| expected.inode += 1);
    assert!(matches!(outcome, UnmountOutcome::Refused { ref step, .. } if step == "inode"), "{outcome:?}");
}

/// A slot whose descriptor names another path refuses.
#[test]
fn a_mismatched_canonical_path_refuses() {
    let fixture = fixture("unmount-turn-path");
    let outcome: UnmountOutcome = run(&fixture, |expected, _calls| expected.canonical_slot = PathBuf::from("/somewhere/else"));
    assert!(matches!(outcome, UnmountOutcome::Refused { ref step, .. } if step == "canonical-path"), "{outcome:?}");
}

/// A row of another source refuses.
#[test]
fn a_mismatched_source_refuses() {
    let fixture = fixture("unmount-turn-source");
    let outcome: UnmountOutcome = run(&fixture, |expected, _calls| expected.source = String::from("pool/clones/two"));
    assert!(matches!(outcome, UnmountOutcome::Refused { ref step, .. } if step == "source"), "{outcome:?}");
}

/// A row on another parent mount refuses.
#[test]
fn a_mismatched_parent_mount_refuses() {
    let fixture = fixture("unmount-turn-parent");
    let outcome: UnmountOutcome = run(&fixture, |expected, _calls| expected.parent_mount_id = 39);
    assert!(matches!(outcome, UnmountOutcome::Refused { ref step, .. } if step == "parent-mount"), "{outcome:?}");
}

/// A live GUID other than the removal's refuses.
#[test]
fn a_mismatched_guid_refuses() {
    let fixture = fixture("unmount-turn-guid");
    let outcome: UnmountOutcome = run(&fixture, |expected, _calls| expected.guid = DatasetGuid::parse("78").expect("the other GUID parses"));
    assert!(matches!(outcome, UnmountOutcome::Refused { ref step, .. } if step == "guid"), "{outcome:?}");
}

/// A second canonical lookup that reaches another mount refuses.
#[test]
fn a_changed_canonical_lookup_refuses() {
    let fixture = fixture("unmount-turn-look-again");
    let mut expected: ExpectedMount = fixture.expected.clone();
    let calls = RecordingCalls { observed: fixture.calls.observed.clone(), mount_id: fixture.mount, ..RecordingCalls::default() };
    *calls.facts.borrow_mut() = Some(row(&fixture.expected));
    *calls.table.borrow_mut() = vec![row(&fixture.expected)];
    let walk = FakeWalk { slot: expected.canonical_slot.clone(), again: 99 };
    let guids = FakeGuids { guid: expected.guid };
    expected.mount_id = 41;
    let request = UnmountRequest { checkout: fixture.checkout.as_path(), slot: "one", nonce: "0123456789abcdef0123456789abcdef", expected: &expected, slot_identity: &fixture.slot_identity, pin: &fixture.pin, cache: &fixture.cache, cache_path: fixture.cache_path.as_path(), boot: "this-boot", calls: &calls, walk: &walk, guids: &guids };
    let outcome = unmount_the_clone(&request);
    assert!(matches!(outcome, UnmountOutcome::Refused { ref step, .. } if step == "canonical-lookup"), "{outcome:?}");
    assert!(!calls.ran(StagingCall::Bind));
}
