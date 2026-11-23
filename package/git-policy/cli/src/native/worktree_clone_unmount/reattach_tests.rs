//! What:
//!  Controls for the checked move back to the slot.
//! Why:
//!  A destination covered by a mount,
//!  a slot folder whose identity moved,
//!  and a proof that fails on either half each stop the move or the claim,
//!  and the arm asserts the step and whether the move ran.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(rollBackToSlot(request).kind).toBe('reattached');
//! ```

use crate::reattach::{roll_back_to_slot, ReattachRequest, ReattachResult};
use crate::slot_walk_calls::SlotWalk;
use crate::staged_folder_identity::StagedFolderIdentity;
use crate::staging_calls::{StagingCall, StagingCallRefusal, StagingCalls};
use crate::staging_calls_control::RecordingCalls;
use std::path::{Path, PathBuf};

/// The walk a control drives.
struct FakeWalk {
    slot: PathBuf,
    covered: bool,
    again: u64,
}

impl SlotWalk for FakeWalk {
    fn open_the_slot(&self, _checkout: &Path, _slot: &str) -> Result<std::fs::File, StagingCallRefusal> {
        return std::fs::File::open(self.slot.as_path()).map_err(|error| return StagingCallRefusal { call: StagingCall::OpenSlot, errno: error.raw_os_error().unwrap_or(0), detail: error.to_string() });
    }

    fn open_the_slot_uncrossed(&self, _checkout: &Path, _slot: &str) -> Result<std::fs::File, StagingCallRefusal> {
        if self.covered {
            return Err(StagingCallRefusal { call: StagingCall::OpenSlot, errno: libc::EXDEV, detail: String::from("a mount covers the slot") });
        }
        return self.open_the_slot(_checkout, _slot);
    }

    fn look_again(&self, _checkout: &Path, _slot: &str) -> Result<u64, StagingCallRefusal> {
        return Ok(self.again);
    }
}

/// The slot,
/// its identity and the row a control reads.
struct Fixture {
    slot: PathBuf,
    staging: std::fs::File,
    identity: StagedFolderIdentity,
    calls: RecordingCalls,
}

/// The fixture slot and the row the move back compares.
fn fixture(name: &str, parent: u64) -> Fixture {
    let root: PathBuf = crate::testing::test_support::fixture_root().join(name);
    let slot: PathBuf = root.join(".claude/worktrees/one");
    std::fs::create_dir_all(slot.as_path()).expect("the fixture slot is made");
    let folder: std::fs::File = std::fs::File::open(slot.as_path()).expect("the fixture slot opens");
    let identity: StagedFolderIdentity = StagedFolderIdentity::of(&folder).expect("the slot's identity is read");
    let calls = RecordingCalls { mount_id: 41, observed: slot.clone(), ..RecordingCalls::default() };
    calls.mount_ids.borrow_mut().extend([40, 41, 41]);
    *calls.facts.borrow_mut() = Some(crate::fs_mount_options::MountFacts {
        mount_id: 41,
        parent_id: parent,
        root: PathBuf::from("/"),
        optional_fields: Vec::new(),
        mount_point: slot.clone(),
        filesystem_type: String::from("zfs"),
        source: String::from("pool/clones/one"),
        mount_options: Vec::new(),
        super_options: Vec::new(),
    });
    let staging: std::fs::File = std::fs::File::open(slot.as_path()).expect("the fixture staging folder opens");
    return Fixture { slot, staging, identity, calls };
}

/// One request over the fixture.
fn request<'a>(fixture: &'a Fixture, identity: &'a StagedFolderIdentity, walk: &'a FakeWalk, checkout: &'a Path) -> ReattachRequest<'a> {
    return ReattachRequest {
        checkout,
        slot: "one",
        clone_source: "pool/clones/one",
        clone_mount_id: 41,
        slot_identity: identity,
        slot_parent_mount_id: 40,
        clone_at_staging: &fixture.staging,
        calls: &fixture.calls,
        walk,
    };
}

/// A free slot receives the clone and both halves of the proof pass.
#[test]
fn a_free_slot_receives_the_clone() {
    let fixture = fixture("reattach-free", 40);
    let walk = FakeWalk { slot: fixture.slot.clone(), covered: false, again: 41 };
    let checkout: PathBuf = fixture.slot.ancestors().nth(3).expect("the checkout is above the slot").to_path_buf();
    let identity: StagedFolderIdentity = fixture.identity;
    let request = request(&fixture, &identity, &walk, checkout.as_path());
    let result: ReattachResult = roll_back_to_slot(&request);
    assert_eq!(result.word(), "reattached");
    assert!(fixture.calls.ran(StagingCall::Move));
}

/// A slot covered by a mount refuses before the move.
#[test]
fn a_covered_slot_refuses_before_the_move() {
    let fixture = fixture("reattach-covered", 40);
    let walk = FakeWalk { slot: fixture.slot.clone(), covered: true, again: 41 };
    let checkout: PathBuf = fixture.slot.ancestors().nth(3).expect("the checkout is above the slot").to_path_buf();
    let identity: StagedFolderIdentity = fixture.identity;
    let request = request(&fixture, &identity, &walk, checkout.as_path());
    let result: ReattachResult = roll_back_to_slot(&request);
    assert!(matches!(result, ReattachResult::Refused { ref step, .. } if step == "open-slot"), "{result:?}");
    assert!(!fixture.calls.ran(StagingCall::Move));
}

/// A slot whose identity moved refuses before the move.
#[test]
fn a_slot_whose_identity_moved_refuses() {
    let fixture = fixture("reattach-identity", 40);
    let walk = FakeWalk { slot: fixture.slot.clone(), covered: false, again: 41 };
    let checkout: PathBuf = fixture.slot.ancestors().nth(3).expect("the checkout is above the slot").to_path_buf();
    let mut identity: StagedFolderIdentity = fixture.identity;
    identity.inode += 1;
    let request = request(&fixture, &identity, &walk, checkout.as_path());
    let result: ReattachResult = roll_back_to_slot(&request);
    assert!(matches!(result, ReattachResult::Refused { ref step, .. } if step == "slot-identity"), "{result:?}");
    assert!(!fixture.calls.ran(StagingCall::Move));
}

/// A proof whose fresh walk reaches another mount refuses the placement.
#[test]
fn a_proof_that_fails_the_walk_refuses() {
    let fixture = fixture("reattach-proof-walk", 40);
    let walk = FakeWalk { slot: fixture.slot.clone(), covered: false, again: 99 };
    let checkout: PathBuf = fixture.slot.ancestors().nth(3).expect("the checkout is above the slot").to_path_buf();
    let identity: StagedFolderIdentity = fixture.identity;
    let request = request(&fixture, &identity, &walk, checkout.as_path());
    let result: ReattachResult = roll_back_to_slot(&request);
    assert!(matches!(result, ReattachResult::Refused { ref step, .. } if step == "post-move-check"), "{result:?}");
}

/// A row whose parent is another mount refuses the placement.
#[test]
fn a_proof_that_fails_the_parent_refuses() {
    let fixture = fixture("reattach-proof-parent", 39);
    let walk = FakeWalk { slot: fixture.slot.clone(), covered: false, again: 41 };
    let checkout: PathBuf = fixture.slot.ancestors().nth(3).expect("the checkout is above the slot").to_path_buf();
    let identity: StagedFolderIdentity = fixture.identity;
    let request = request(&fixture, &identity, &walk, checkout.as_path());
    let result: ReattachResult = roll_back_to_slot(&request);
    assert!(matches!(result, ReattachResult::Refused { ref step, .. } if step == "post-move-check"), "{result:?}");
}

/// What:
///  A busy unmount whose reattach may not happen:
///  the holder forces `EBUSY`,
/// and a mount over the slot makes the destination
///  open refuse before the move.
/// Why:
///  The retained state must be reached by the kernel's busy answer and the
///  destination's `EXDEV`,
/// not by a timed guess,
///  so the test starts and ends its own holder and mounts the cover itself.
#[test]
fn a_busy_unmount_whose_slot_is_covered_refuses_the_reattach() {
    let test: &str = "worktree_clone_unmount::reattach::tests::a_busy_unmount_whose_slot_is_covered_refuses_the_reattach";
    if !in_a_mount_namespace("STAGED_REATTACH_MAY_NOT_INSIDE", test) {
        return;
    }
    let root: std::path::PathBuf = crate::testing::test_support::fixture_root().join("reattach-may-not");
    let checkout: std::path::PathBuf = root.join("checkout");
    let slot: std::path::PathBuf = checkout.join(".claude/worktrees/one");
    let parent: std::path::PathBuf = checkout.join(".claude/worktrees");
    std::fs::create_dir_all(slot.as_path()).expect("the slot is made");
    let cache_path: std::path::PathBuf = root.join("cache");
    let name: String = crate::staging_folder_name::staging_folder_name(crate::worktree_clone_mount_protocol::DatasetGuid::parse("71").expect("the GUID parses"));
    let staging: std::path::PathBuf = cache_path.join(name.as_str());
    std::fs::create_dir_all(staging.as_path()).expect("the staging folder is made");
    let cache: std::fs::File = std::fs::File::open(cache_path.as_path()).expect("the cache opens");
    let folder: std::fs::File = std::fs::File::open(staging.as_path()).expect("the staging folder opens");
    let bind: std::fs::File = crate::staging_calls::LiveStagingCalls.bind_the_folder(&folder, &cache, std::ffi::OsStr::new(name.as_str())).expect("the folder binds");
    crate::staging_calls::LiveStagingCalls.make_private(&bind).expect("the bind is private");
    let clone: std::fs::File = mount_a_tmpfs_at(root.join("clone").as_path());
    crate::staging_calls::LiveStagingCalls.move_clone(&clone, &bind).expect("the tmpfs moves onto the bind");
    drop(clone);
    drop(bind);
    let slot_identity = crate::staged_folder_identity::StagedFolderIdentity::of(&std::fs::File::open(slot.as_path()).expect("the empty slot opens")).expect("the slot identity reads");
    let parent_file: std::fs::File = std::fs::File::open(parent.as_path()).expect("the slot parent opens");
    let parent_mount: u64 = crate::staging_calls::LiveStagingCalls.mount_id(&parent_file).expect("the slot parent's mount reads");
    let staged: std::fs::File = crate::staging_calls::LiveStagingCalls.open_by_name(&cache, std::ffi::OsStr::new(name.as_str())).expect("the staged clone opens");
    let mut holder: std::process::Child = std::process::Command::new("sh").args(["-c", format!("cd {} && exec sleep 60", staging.display()).as_str()]).spawn().expect("the holder starts");
    let until = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while std::time::Instant::now() < until && std::fs::read_link(format!("/proc/{}/cwd", holder.id())).ok() != std::fs::canonicalize(staging.as_path()).ok() {
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let busy = crate::staged_unmount::unmount_the_staged_mount(&crate::staged_unmount::HeldUnmount { cache: &cache, folder: &folder, name: name.as_str(), clone_source: "tmpfs", calls: &crate::staging_calls::LiveStagingCalls });
    assert!(matches!(busy, crate::staged_unmount::HeldOutcome::Retained { errno: libc::EBUSY, .. }), "the holder forces EBUSY: {busy:?}");
    mount_a_tmpfs_at(slot.as_path());
    let refused = roll_back_to_slot(&crate::reattach::ReattachRequest {
        checkout: checkout.as_path(),
        slot: "one",
        clone_source: "tmpfs",
        clone_mount_id: crate::staging_calls::LiveStagingCalls.mount_id(&staged).expect("the staged clone's mount reads"),
        slot_identity: &slot_identity,
        slot_parent_mount_id: parent_mount,
        clone_at_staging: &staged,
        calls: &crate::staging_calls::LiveStagingCalls,
        walk: &crate::slot_walk_calls::LiveSlotWalk,
    });
    assert!(matches!(refused, ReattachResult::Refused { ref step, .. } if step == "open-slot"), "a covered slot refuses before the move: {refused:?}");
    let _ = holder.kill();
    let _ = holder.wait();
}

/// Mount a fresh tmpfs and answer a descriptor on it.
fn mount_a_tmpfs_at(target: &std::path::Path) -> std::fs::File {
    use std::os::unix::ffi::OsStrExt;
    std::fs::create_dir_all(target).expect("the mount point is made");
    let at = std::ffi::CString::new(target.as_os_str().as_bytes()).expect("a path holds no NUL");
    let kind = std::ffi::CString::new("tmpfs").expect("no NUL in a type");
    let data = std::ffi::CString::new("size=64k,nr_inodes=16,mode=0700").expect("no NUL in an option");
    // SAFETY: every pointer is this call's own and outlives it.
    let mounted: libc::c_int = unsafe { libc::mount(kind.as_ptr(), at.as_ptr(), kind.as_ptr(), libc::MS_NOSUID | libc::MS_NODEV, data.as_ptr().cast::<libc::c_void>()) };
    assert_eq!(mounted, 0, "the fixture tmpfs mounts: {}", std::io::Error::last_os_error());
    return std::fs::File::open(target).expect("the mounted folder opens");
}

/// Whether this run is the inner half of a private user and mount namespace.
#[allow(clippy::disallowed_methods, reason = "starts this test binary, not the wrapper, under `unshare`")]
fn in_a_mount_namespace(variable: &str, test: &str) -> bool {
    if std::env::var_os(variable).is_some() {
        return true;
    }
    let done = std::process::Command::new("unshare")
        .args(["--user", "--map-root-user", "--mount", "--propagation", "private"])
        .arg(std::env::current_exe().expect("this test binary"))
        .args(["--exact", test, "--nocapture"])
        .env(variable, "1")
        .output()
        .expect("the inner half runs");
    let said: String = String::from_utf8_lossy(done.stdout.as_slice()).to_string() + String::from_utf8_lossy(done.stderr.as_slice()).as_ref();
    assert!(done.status.success(), "the inner half failed:\n{said}");
    assert!(said.contains("1 passed"), "the inner half did not run:\n{said}");
    return false;
}
