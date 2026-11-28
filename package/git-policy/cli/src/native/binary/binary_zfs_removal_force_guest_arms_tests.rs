//! What:
//!  The three arms of the forced removal that need a real pool and a real
//!  mount:
//!  a clone an interrupted removal left on the staging folder,
//!  the forced run's live proof and its snapshot batch,
//!  and a clone the wrapper cannot prove.
//! Why:
//!  Each is a fact only a kernel and a pool answer:
//!  a dataset cannot be destroyed while it is mounted,
//!  a snapshot goes before the head that holds it,
//!  and a reused dataset name answers a GUID the add's record did not record.
//!  The host controls answer every seam with a scripted value,
//!  so only these arms show what a person's command does to the guest's own
//!  datasets,
//!  and each reads the pool and the records after the command rather than the
//!  command's words alone.
//!  They run where `CLI_GIT_ZFS_REMOVAL_FORCE_GUEST` names the guest,
//!  printing `SKIP` everywhere else.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const said = arm.wrapperGit(['worktree', 'remove', '--force', '--force', shown(arm.place.slot)]);
//! expect(said).toContain('two forces were given, so this removal destroyed the dataset');
//! ```

use super::zfs_removal_force_guest_fixture::{
    Arm,
    OWN_SNAPSHOT_PREFIX,
    SLOT,
    in_the_guest,
    plant_an_unreadable_plan,
    shown,
    the_record_home,
};
use super::zfs_sweep_guest::{
    DESIRED_STATE,
    NONCE,
    ORIGIN_PREFIX,
    command_ok,
    mount_request,
    zfs,
    zfs_ok,
};
use git_policy_cli::worktree_clone_binding_record::registration_home::binding_name;
use git_policy_cli::worktree_clone_mount_protocol::DatasetGuid;
use git_policy_cli::worktree_clone_unmount::staged_folder_identity::StagedFolderIdentity;
use git_policy_cli::worktree_clone_unmount::staged_folder_record::write_record;
use git_policy_cli::worktree_clone_unmount::staged_folder_record_text::StagedFolderRecord;
use git_policy_cli::worktree_clone_unmount::staging_calls::{
    BOOT_ID_PATH,
    LiveStagingCalls,
    StagingCalls,
};
use git_policy_cli::worktree_clone_unmount::staging_folder_name::staging_folder_name;
use git_policy_cli::worktree_clone_unmount::user_cache_pin::{
    UserCachePin,
    open_cache,
    write_pin,
};
use std::ffi::OsStr;
use std::os::unix::fs::MetadataExt;
use std::path::{
    Path,
    PathBuf,
};
use std::process::Child;

/// The folder the handler files its receipts in,
/// which is the release's own `RESULTS_DIRECTORY`.
const THE_RECEIPTS_FOLDER: &str = "/run/git-worktree-clone-mount-results";

/// The folder the pin record lives in,
/// which is the release's own `USER_CACHE_PIN_PATH`'s parent.
const THE_PIN_FOLDER: &str = "/var/lib/monochromatic-git-policy-cli";

/// What:
///  Arm 1:
///  a clone an interrupted removal left mounted on its staging folder.
/// Why:
///  The project decided that one force unmounts such a clone there and then
///  destroys it,
///  and that the forced run acts on the handler's own answer to the unmount it
///  asks for,
///  so the arm first shows the pool refusing the destroy while the clone is
///  mounted,
///  then the forced command's report and the pool's answer after the handler's
///  receipt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = arm.wrapperGit(['worktree', 'remove', '--force', slot]);
/// expect(arm.mountsOfTheClone()).toEqual([]);
/// ```
#[test]
fn zfs_removal_force_guest_the_staging_folder_arm() {
    if !in_the_guest() {
        return;
    }
    let arm: Arm = Arm::make(1);
    arm.attach_at_the_slot();
    arm.snapshot(format!("{OWN_SNAPSHOT_PREFIX}a").as_str());
    let name: String = staging_folder_name(arm.clone_guid);
    let (staging, cache): (PathBuf, std::fs::File) = the_pinned_cache(&arm, name.as_str());
    let staging_file: std::fs::File =
        std::fs::File::open(staging.as_path()).expect("the staging folder opens");
    let identity: StagedFolderIdentity =
        StagedFolderIdentity::of(&staging_file).expect("the staging folder's identity reads");
    let (handle_type, handle): (i32, Vec<u8>) = LiveStagingCalls
        .name_handle(&staging_file)
        .expect("the kernel names the staging folder by a handle");
    let (namespace_device, namespace_inode): (u64, u64) = LiveStagingCalls
        .namespace_pair()
        .expect("this process's mount namespace reads");
    let boot: String = std::fs::read_to_string(BOOT_ID_PATH)
        .expect("the guest's boot identifier reads")
        .trim()
        .to_string();
    // The record is written before the bind, as the handler's own turn writes it.
    let record: StagedFolderRecord = StagedFolderRecord {
        component: name.clone(),
        dataset: arm.clone.clone(),
        guid: arm.clone_guid,
        identity,
        handle_type,
        handle,
        nonce: String::from(NONCE),
        namespace_device,
        namespace_inode,
        boot,
    };
    write_record(&cache, &record).expect("the staged-folder record is written");
    let bind: std::fs::File = LiveStagingCalls
        .bind_the_folder(&staging_file, &cache, OsStr::new(name.as_str()))
        .expect("the staging folder is bound onto the cache");
    LiveStagingCalls
        .make_private(&bind)
        .expect("the bind is made private");
    let slot_file: std::fs::File =
        std::fs::File::open(arm.place.slot.as_path()).expect("the slot opens");
    LiveStagingCalls
        .move_clone(&slot_file, &bind)
        .expect("the clone's mount is moved onto the staging folder");
    // Every descriptor this arm holds is released, because a descriptor on a mount holds that mount busy and the
    // handler's own unmount is what this arm is about.
    drop(slot_file);
    drop(bind);
    drop(staging_file);
    drop(cache);
    let rows: Vec<String> = arm.mounts_of_the_clone();
    assert_eq!(
        rows.len(),
        1,
        "the clone stands on the staging folder and nowhere else: {rows:?}"
    );
    // The pool's own answer: a mounted dataset is not destroyed.
    let refused: (bool, String, String) = zfs(
        arm.sbin.as_str(),
        "zfs",
        &["destroy", arm.clone.as_str()],
        None,
    );
    assert!(
        !refused.0,
        "the pool destroyed a clone mounted on its staging folder: {}",
        refused.1
    );
    println!(
        "REMOVAL-FORCE arm=1 mounted-refuses-the-destroy said={}",
        refused.2.trim()
    );

    let sweeper: Child = arm.start_the_sweeper();
    let removed: std::process::Output = arm.wrapper_git(&[
        "worktree",
        "remove",
        "--force",
        shown(arm.place.slot.as_path()),
    ]);
    arm.stop_the_sweeper(sweeper);
    let said: String = Arm::said(&removed);
    assert_eq!(
        removed.status.code(),
        Some(0),
        "one force unmounts the clone there and destroys it: {said}"
    );
    assert!(
        said.contains(&format!(
            "the ZFS worktree clone at `{}` (dataset `{}`) is destroyed",
            shown(arm.place.slot.as_path()),
            arm.clone
        )),
        "the loss is printed in the same invocation: {said}"
    );
    assert!(
        said.contains(&format!(
            "destroyed snapshot {}@{OWN_SNAPSHOT_PREFIX}a",
            arm.clone
        )),
        "the clone's own snapshot went with it: {said}"
    );
    assert!(
        said.contains("the clone's head destroy answered and its absence is proved."),
        "the head went after the batch: {said}"
    );
    assert!(
        arm.mounts_of_the_clone().is_empty(),
        "the clone's mount is gone: {:?}",
        arm.mounts_of_the_clone()
    );
    assert!(
        mount_of_folder(staging.as_path()).is_none(),
        "the bind over the staging folder is gone too"
    );
    assert!(!arm.exists(arm.clone.as_str()), "the clone dataset is gone");
    let receipt: String = the_receipt_of(arm.clone_guid);
    assert!(
        receipt.contains("done") && receipt.contains("unmount"),
        "the handler's own receipt is the answer the forced run verified: {receipt}"
    );
    println!(
        "REMOVAL-FORCE arm=1 one-force=unmounted-there-then-destroyed receipt=done staging=gone"
    );
    arm.tear_down();
}

/// What:
///  Arm 2:
///  the forced run's live proof and its snapshot batch against a real clone.
/// Why:
///  The owner's answer makes a forced run destroy what the pool lists for its
///  clone,
///  so the arm reads the pool's own snapshot list before the command and
///  compares it, in the pool's order, with the names the report records as
///  destroyed,
///  and then reads the pool after the head went.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(destroyed).toEqual(poolSnapshotsBefore);
/// ```
#[test]
fn zfs_removal_force_guest_the_snapshot_batch_arm() {
    if !in_the_guest() {
        return;
    }
    let arm: Arm = Arm::make(2);
    arm.attach_at_the_slot();
    arm.snapshot(format!("{OWN_SNAPSHOT_PREFIX}a").as_str());
    arm.snapshot(format!("{OWN_SNAPSHOT_PREFIX}b").as_str());
    let plan: PathBuf = plant_an_unreadable_plan(arm.place.common.as_path(), arm.clone_guid);
    let before: Vec<String> = arm.the_snapshots();
    assert_eq!(
        before,
        vec![
            format!("{OWN_SNAPSHOT_PREFIX}a"),
            format!("{OWN_SNAPSHOT_PREFIX}b")
        ],
        "the pool lists the clone's own snapshots before the run"
    );
    let sweeper: Child = arm.start_the_sweeper();
    let removed: std::process::Output = arm.wrapper_git(&[
        "worktree",
        "remove",
        "--force",
        shown(arm.place.slot.as_path()),
    ]);
    arm.stop_the_sweeper(sweeper);
    let said: String = Arm::said(&removed);
    assert_eq!(
        removed.status.code(),
        Some(0),
        "one force goes past a plan it cannot read and destroys the clone: {said}"
    );
    assert!(
        said.contains(&format!(
            "a force was given, so this removal went past the destruction plan at {}, which could not be read",
            plan.display()
        )),
        "the plan the run went past is named: {said}"
    );
    let destroyed: Vec<String> = destroyed_in(&said);
    assert_eq!(
        destroyed, before,
        "every snapshot the pool listed is recorded as destroyed, in the pool's own order: {said}"
    );
    assert!(
        said.contains("the clone's head destroy answered and its absence is proved."),
        "the head went after the batch: {said}"
    );
    assert!(!arm.exists(arm.clone.as_str()), "the clone dataset is gone");
    assert!(
        arm.the_snapshots().is_empty(),
        "the pool lists no snapshot under the clone: {:?}",
        arm.the_snapshots()
    );
    assert!(!plan.exists(), "the unreadable plan's file is gone");
    assert_eq!(arm.plan_entries(), 0, "no plan stands after the removal");
    assert!(
        !the_record_home(arm.place.common.as_path())
            .join(binding_name(SLOT))
            .exists(),
        "the binding record went with the other records"
    );
    println!(
        "REMOVAL-FORCE arm=2 live-proof=pool-list snapshots-before={before:?} destroyed={destroyed:?} head=gone"
    );
    arm.tear_down();
}

/// What:
///  Arm 3:
///  a clone the wrapper cannot prove,
///  because the name its record holds answers another GUID.
/// Why:
///  The owner answered that two forces destroy the dataset the record names even
///  when the pool's answer does not match it,
///  and that nothing can prove afterwards which clone it was.
///  The arm makes the pool's answer differ from the record's by reusing the
///  name the way a person or a script can,
///  and then reads both GUIDs out of the refusals and out of the report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(said).toContain(`the record held ${recorded} and the pool answered ${live}`);
/// ```
#[test]
fn zfs_removal_force_guest_a_clone_the_wrapper_cannot_prove_arm() {
    if !in_the_guest() {
        return;
    }
    let arm: Arm = Arm::make(3);
    let recorded: DatasetGuid = arm.clone_guid;
    // The name the record holds is reused by another object: the record keeps its GUID and the pool answers a new one.
    let again: String = format!(
        "{}@{ORIGIN_PREFIX}{NONCE}-again",
        arm.place.checkout_dataset
    );
    zfs_ok(arm.sbin.as_str(), &["snapshot", again.as_str()]);
    zfs_ok(
        arm.sbin.as_str(),
        &["destroy", "-r", "-f", arm.clone.as_str()],
    );
    zfs_ok(
        arm.sbin.as_str(),
        &["clone", again.as_str(), arm.clone.as_str()],
    );
    zfs_ok(
        arm.sbin.as_str(),
        &[
            "set",
            format!("{DESIRED_STATE}={}", mount_request(NONCE)).as_str(),
            arm.clone.as_str(),
        ],
    );
    let live: DatasetGuid =
        super::zfs_slot_state_guest_fixture::guid_of(arm.sbin.as_str(), arm.clone.as_str());
    assert_ne!(
        recorded, live,
        "the pool answers another object under the name the record holds"
    );
    arm.attach_at_the_slot();
    arm.snapshot(format!("{OWN_SNAPSHOT_PREFIX}a").as_str());
    let snapshots: Vec<String> = arm.the_snapshots();
    let sweeper: Child = arm.start_the_sweeper();
    let none: std::process::Output =
        arm.wrapper_git(&["worktree", "remove", shown(arm.place.slot.as_path())]);
    let one: std::process::Output = arm.wrapper_git(&[
        "worktree",
        "remove",
        "--force",
        shown(arm.place.slot.as_path()),
    ]);
    arm.stop_the_sweeper(sweeper);
    for (forces, answer) in [("none", &none), ("one", &one)] {
        let said: String = Arm::said(answer);
        assert_eq!(
            answer.status.code(),
            Some(1),
            "with {forces} force the clone cannot be proved and nothing is destroyed: {said}"
        );
        assert!(
            said.contains(&format!("the clone of the slot `{SLOT}` cannot be proved")),
            "the refusal says what it could not prove: {said}"
        );
        assert!(
            said.contains(recorded.file_name().as_str())
                && said.contains(live.file_name().as_str()),
            "the refusal names the record's GUID and the pool's: {said}"
        );
        assert!(
            said.contains(&format!("`git worktree remove --force --force {SLOT}`")),
            "the refusal names the two forces that pass it: {said}"
        );
        assert!(
            arm.exists(arm.clone.as_str()),
            "nothing was destroyed with {forces} force"
        );
        assert_eq!(
            arm.the_snapshots(),
            snapshots,
            "no snapshot went with {forces} force"
        );
        assert_eq!(arm.plan_entries(), 0, "no plan was written");
    }
    let two_forces_sweeper: Child = arm.start_the_sweeper();
    let two: std::process::Output = arm.wrapper_git(&[
        "worktree",
        "remove",
        "--force",
        "--force",
        shown(arm.place.slot.as_path()),
    ]);
    arm.stop_the_sweeper(two_forces_sweeper);
    let said: String = Arm::said(&two);
    assert_eq!(
        two.status.code(),
        Some(0),
        "two forces destroy the dataset the record names: {said}"
    );
    assert!(
        said.contains(&format!(
            "two forces were given, so this removal destroyed the dataset `{}`",
            arm.clone
        )),
        "the report names the dataset it destroyed: {said}"
    );
    assert!(
        said.contains(&format!(
            "the record held `{}` and the pool answered `{}`",
            recorded.file_name(),
            live.file_name()
        )),
        "the report names both GUIDs: {said}"
    );
    assert!(
        !arm.exists(arm.clone.as_str()),
        "the dataset the record named is gone"
    );
    println!(
        "REMOVAL-FORCE arm=3 refused=none-and-one destroyed=two recorded-guid={} pool-guid={} destroyed-snapshots={snapshots:?}",
        recorded.file_name(),
        live.file_name()
    );
    arm.tear_down();
}

/// What:
///  A dataset mounted at the account's cache, pinned as the route pins it,
///  and the empty staging folder the removal derives under it.
/// Why:
///  The handler's staged unmount checks the cache's mount against the pin
///  before either `umount2`,
///  and the folder it unmounts is the one the clone's GUID derives,
///  so an arm that made either differently would prove nothing about the
///  route.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [staging, cache] = thePinnedCache(arm, stagingFolderName(guid));
/// ```
fn the_pinned_cache(
    arm: &Arm,
    name: &str,
) -> (PathBuf, std::fs::File) {
    let dataset: String = format!("{}/cache", arm.pool);
    zfs_ok(
        arm.sbin.as_str(),
        &["create", "-o", "mountpoint=legacy", dataset.as_str()],
    );
    let cache_path: PathBuf = PathBuf::from("/root/.cache");
    std::fs::create_dir_all(cache_path.as_path()).expect("the account's cache folder is there");
    command_ok(
        "mount",
        &["-t", "zfs", dataset.as_str(), shown(cache_path.as_path())],
    );
    let facts: std::fs::Metadata =
        std::fs::metadata(cache_path.as_path()).expect("the cache's own facts read");
    let pin: UserCachePin = UserCachePin {
        source: dataset,
        root_inode: facts.ino(),
        device: facts.dev(),
    };
    std::fs::create_dir_all(Path::new(THE_PIN_FOLDER)).expect("the pin's folder is made");
    let folder: std::fs::File =
        std::fs::File::open(Path::new(THE_PIN_FOLDER)).expect("the pin's folder opens");
    write_pin(&folder, &pin).expect("the pin is written");
    drop(folder);
    let cache: std::fs::File = open_cache(Path::new("/root")).expect("the cache opens");
    let staging: PathBuf = cache_path.join(name);
    std::fs::create_dir_all(staging.as_path()).expect("the staging folder is made");
    std::fs::set_permissions(
        staging.as_path(),
        std::os::unix::fs::PermissionsExt::from_mode(0o700),
    )
    .expect("the staging folder holds the design's mode");
    return (staging, cache);
}

/// The one mount row whose mount point is this folder,
/// which is how an arm proves a bind left.
fn mount_of_folder(folder: &Path) -> Option<String> {
    return super::zfs_sweep_guest::mount_row(folder);
}

/// Every snapshot name a removal's report records as destroyed, in its order.
fn destroyed_in(said: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for line in said.lines() {
        let Some(name) = line.strip_prefix("  destroyed snapshot ") else {
            continue;
        };
        let Some((_, snapshot)) = name.split_once('@') else {
            continue;
        };
        names.push(String::from(snapshot));
    }
    return names;
}

/// What:
///  The receipt the handler filed for one clone,
///  as the guest's own bytes.
/// Why:
///  The forced run verifies this answer before it destroys anything,
///  so the arm reads the file the handler wrote rather than trusting the
///  wrapper's own sentence about it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const receipt = theReceiptOf(guid);
/// ```
fn the_receipt_of(guid: DatasetGuid) -> String {
    let path: PathBuf = Path::new(THE_RECEIPTS_FOLDER).join(guid.file_name());
    return std::fs::read_to_string(path.as_path()).unwrap_or_else(|error| {
        panic!(
            "the handler's receipt at {} reads ({error}); the folder holds {:?}",
            path.display(),
            std::fs::read_dir(THE_RECEIPTS_FOLDER).map(|entries| {
                return entries
                    .filter_map(|entry| return entry.ok())
                    .map(|entry| return entry.file_name())
                    .collect::<Vec<std::ffi::OsString>>();
            })
        )
    });
}
