//! Controls for the clone domain:
//! which pairs of paths share data blocks,
//! and the refusal each pair that does not is given.

use super::*;
use crate::fs_kind::may_clone;
use crate::test_support::{fixture, remove};
use crate::worktree_add_plan::AddPlan;
use crate::worktree_ladder_test_support::{main_identity, plan_to};
use crate::worktree_snapshot_source::clone_refusal;

/// The dataset the design gives the checkout:
/// `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
/// section "Dataset and path layout".
const CHECKOUT_DATASET: &str = "zpcachyos/ROOT/cos/home/monochromatic-git-checkout";

/// The container the design puts every managed clone under,
/// so each clone is a dataset of its own.
const CLONE_DATASET: &str = "zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets/topic";

/// The device number this host gives the checkout's dataset;
/// a renamed dataset or a clone's own dataset is another device,
/// which is the fact that pins a dataset name to one mount.
const CHECKOUT_DEVICE: u64 = 57;

/// The device number of a mounted clone's dataset,
/// which the design gives a device of its own because it is a dataset
///  of its own.
const CLONE_DEVICE: u64 = 64;

/// Two stand-in device numbers for btrfs mounts,
/// whose only fact read is whether they agree.
const BTRFS_DEVICE: u64 = 40;

/// Two stand-in device numbers for overlay mounts,
/// whose only fact read is whether they agree.
const OVERLAY_DEVICE: u64 = 700;

/// What:
///  The clone facts one ZFS path gives:
/// its dataset as the mount source the kernel reports,
/// and the device of that
///  dataset's own mount.
/// Why:
///   The real mounts belong to the step's VM arm,
/// whose guest fixture is the only pool this repository may write to;
/// the decision
///  itself is what the host proves,
/// so its answers are supplied here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const side = (path: string, dataset: string, device: number) => ({ path, kind: 'zfs', device, mountSource: dataset });
/// ```
fn zfs_side(path: &str, dataset: &str, device: u64) -> CloneSide {
    return CloneSide { path: PathBuf::from(path), kind: FilesystemKind::Zfs, device: Some(device), mount_source: Some(String::from(dataset)) };
}

/// A main checkout and a slot beside it,
/// in one dataset:
/// the same dataset name and the same device,
/// so the clones share the source's blocks.
#[test]
fn zfs_two_paths_in_one_dataset_are_one_clone_domain() {
    let checkout: CloneSide = zfs_side("/home/user/Monochromatic", CHECKOUT_DATASET, CHECKOUT_DEVICE);
    let slot: CloneSide = zfs_side("/home/user/Monochromatic/.claude/worktrees/topic", CHECKOUT_DATASET, CHECKOUT_DEVICE);
    assert_eq!(clone_domain_refusal(&checkout, &slot), None);
}

/// A mounted clone at the destination is another dataset,
/// so a reflink into that slot would leave the checkout's dataset,
/// and the refusal names both datasets.
#[test]
fn zfs_a_clone_mounted_at_the_destination_is_refused() {
    let checkout: CloneSide = zfs_side("/home/user/Monochromatic", CHECKOUT_DATASET, CHECKOUT_DEVICE);
    let mounted_clone: CloneSide = zfs_side("/home/user/Monochromatic/.claude/worktrees/topic", CLONE_DATASET, CLONE_DEVICE);
    let reason: String = clone_domain_refusal(&checkout, &mounted_clone).expect("a mount of another dataset is no clone domain");
    assert!(reason.contains(CHECKOUT_DATASET) && reason.contains(CLONE_DATASET), "{reason}");
    assert!(reason.contains("only inside one dataset"), "the refusal says what a reflink needs: {reason}");
}

/// A mounted clone used as the source for a slot in the checkout dataset is refused the same way,
/// in the other direction.
#[test]
fn zfs_a_clone_source_for_a_checkout_slot_is_refused() {
    let mounted_clone: CloneSide = zfs_side("/home/user/Monochromatic/.claude/worktrees/topic", CLONE_DATASET, CLONE_DEVICE);
    let slot: CloneSide = zfs_side("/home/user/Monochromatic/.claude/worktrees/next", CHECKOUT_DATASET, CHECKOUT_DEVICE);
    let reason: String = clone_domain_refusal(&mounted_clone, &slot).expect("a clone's dataset is no checkout dataset");
    assert!(reason.contains(CLONE_DATASET) && reason.contains(CHECKOUT_DATASET), "{reason}");
}

/// What:
///  A dataset name alone is not the dataset:
/// the device that name's mount has must agree too.
/// Why:
///   The device is what pins the name to one mounted dataset,
/// so a pair that agrees on the name and not on the mount is
///  refused rather than trusted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(cloneDomainRefusal(side(a, name, checkoutDevice), side(b, name, checkoutDevice + 1))).not.toBe(undefined);
/// ```
#[test]
fn zfs_one_dataset_name_on_two_devices_is_refused() {
    let first: CloneSide = zfs_side("/home/user/Monochromatic", CHECKOUT_DATASET, CHECKOUT_DEVICE);
    let second: CloneSide = zfs_side("/home/user/Monochromatic/.claude/worktrees/topic", CHECKOUT_DATASET, CHECKOUT_DEVICE + 1);
    let reason: String = clone_domain_refusal(&first, &second).expect("one name on two devices proves nothing");
    assert!(reason.contains(CHECKOUT_DEVICE.to_string().as_str()) && reason.contains((CHECKOUT_DEVICE + 1).to_string().as_str()), "the refusal names both devices: {reason}");
}

/// A side whose dataset the mount table cannot name,
/// or whose device cannot be read,
/// is unproved,
/// and an unproved pair keeps Git's own checkout.
#[test]
fn zfs_a_side_that_cannot_be_proven_is_refused() {
    let checkout: CloneSide = zfs_side("/home/user/Monochromatic", CHECKOUT_DATASET, CHECKOUT_DEVICE);
    let unnamed: CloneSide = CloneSide { mount_source: None, ..zfs_side("/home/user/Monochromatic/.claude/worktrees/topic", CHECKOUT_DATASET, CHECKOUT_DEVICE) };
    let unread: CloneSide = CloneSide { device: None, ..zfs_side("/home/user/Monochromatic/.claude/worktrees/other", CHECKOUT_DATASET, CHECKOUT_DEVICE) };
    assert!(clone_domain_refusal(&checkout, &unnamed).expect("an unnamed dataset is unproved").contains("mount table"));
    assert!(clone_domain_refusal(&checkout, &unread).expect("an unread device is unproved").contains("device"));
}

/// A ZFS path and a path on another kind of filesystem never share one clone domain,
/// whatever the other filesystem is.
#[test]
fn zfs_and_another_filesystem_are_different_filesystems() {
    let checkout: CloneSide = zfs_side("/home/user/Monochromatic", CHECKOUT_DATASET, CHECKOUT_DEVICE);
    let overlay: CloneSide = CloneSide { path: PathBuf::from("/fixtures/topic"), kind: FilesystemKind::Overlayfs, device: Some(CHECKOUT_DEVICE), mount_source: None };
    let reason: String = clone_domain_refusal(&checkout, &overlay).expect("a ZFS path and an overlay path are no one domain");
    assert!(reason.contains("/home/user/Monochromatic") && reason.contains("/fixtures/topic"), "{reason}");
    assert!(reason.contains("different filesystems"), "the refusal names the cause and not a missing dataset: {reason}");

    let other: CloneSide = CloneSide { path: PathBuf::from("/fixtures/topic"), kind: FilesystemKind::Other, device: Some(CHECKOUT_DEVICE), mount_source: None };
    assert!(clone_domain_refusal(&overlay, &other).expect("an overlay path and an XFS path are no one domain").contains("different filesystems"));
}

/// btrfs keeps the rule it had:
/// one filesystem is one mount source,
/// and another mount source is another filesystem.
#[test]
fn btrfs_one_mount_source_is_one_clone_domain() {
    let source: CloneSide = CloneSide { path: PathBuf::from("/fixtures/repo"), kind: FilesystemKind::Btrfs, device: Some(BTRFS_DEVICE), mount_source: Some(String::from("/dev/sda2")) };
    let same: CloneSide = CloneSide { path: PathBuf::from("/fixtures/topic"), kind: FilesystemKind::Btrfs, device: Some(BTRFS_DEVICE), mount_source: Some(String::from("/dev/sda2")) };
    let other: CloneSide = CloneSide { path: PathBuf::from("/fixtures/elsewhere"), kind: FilesystemKind::Btrfs, device: Some(BTRFS_DEVICE + 1), mount_source: Some(String::from("/dev/sdb2")) };
    assert_eq!(clone_domain_refusal(&source, &same), None);
    assert!(clone_domain_refusal(&source, &other).expect("two btrfs filesystems are two domains").contains("different filesystems"));
}

/// An overlay or XFS pair is one device,
/// and a pair whose device cannot be read is unproved.
#[test]
fn an_overlay_pair_is_one_device() {
    let source: CloneSide = CloneSide { path: PathBuf::from("/fixtures/repo"), kind: FilesystemKind::Overlayfs, device: Some(OVERLAY_DEVICE), mount_source: None };
    let same: CloneSide = CloneSide { path: PathBuf::from("/fixtures/topic"), kind: FilesystemKind::Overlayfs, device: Some(OVERLAY_DEVICE), mount_source: None };
    let other: CloneSide = CloneSide { path: PathBuf::from("/elsewhere/topic"), kind: FilesystemKind::Overlayfs, device: Some(OVERLAY_DEVICE + 1), mount_source: None };
    assert_eq!(clone_domain_refusal(&source, &same), None);
    assert!(clone_domain_refusal(&source, &other).expect("two devices are two filesystems").contains("different filesystems"));
    let unread: CloneSide = CloneSide { device: None, ..same };
    assert!(clone_domain_refusal(&source, &unread).expect("an unread device is unproved").contains("cannot both be examined"));
}

/// What:
///  The clones land in the destination when something is already there and in its parent otherwise.
/// Why:
///   The destination does not exist yet before Git runs,
/// so its parent's filesystem receives the bytes;
/// a mounted clone at
///  the destination is exactly the case where it does exist,
/// and the bytes then land in the clone's dataset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(cloneWriteTarget([take, parent])).toBe(take); expect(cloneWriteTarget([absent, parent])).toBe(parent);
/// ```
#[test]
fn the_clones_land_in_the_destination_when_it_is_there_and_in_its_parent_otherwise() {
    let root: PathBuf = fixture("clone-write-target");
    let taken: PathBuf = root.join("taken");
    std::fs::create_dir(taken.as_path()).expect("a slot something already occupies");
    assert_eq!(clone_write_target((taken.as_path(), root.as_path())), taken);
    let absent: PathBuf = root.join("absent");
    assert_eq!(clone_write_target((absent.as_path(), root.as_path())), root);
    remove(root.as_path());
}

/// What:
///  The rung asks the filesystem that would receive the clones,
/// so a destination that occupies its slot is what the refusal names,
/// and a parent that cannot clone on its own does not hide it.
/// Why:
///   Without this control the only test of the destination rule asks the helper directly,
/// so
///  asking the parent again leaves every other control green:
/// `/proc` is a mount of its own inside the container's root filesystem
///  and announces no clone,
/// while the parent a reverted rung would ask announces one here,
/// so the
///  name the refusal gives is the whole difference.
/// The assertion is the whole phrase and not the path,
/// because another refusal can name a path that merely
///  begins with `/proc`:
/// a fixture folder below `/process-<pid>/` does,
/// and a host whose root is one ZFS dataset answers a
///  reverted rung with the dataset refusal that names it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(cloneRefusal(mainIdentity(root), planTo('/proc'))).toContain('/proc would receive the clones');
/// ```
#[test]
fn the_clone_refusal_asks_the_filesystem_that_would_receive_the_clones() {
    let root: PathBuf = fixture("clone-refusal-write-target");
    let plan: AddPlan = plan_to(PathBuf::from("/proc"));
    let reason: String = clone_refusal(&main_identity(root.as_path()), &plan).expect("procfs announces no reflink, so it is no clone domain");
    assert!(reason.contains("/proc would receive the clones"), "the refusal names the destination as the path that would receive the clones: {reason}");
    assert!(!reason.contains("different filesystems"), "the destination is refused as a filesystem that cannot clone and not as a domain beside its parent: {reason}");
    remove(root.as_path());
}

/// What:
///  The facts are read from the kernel for a real pair:
/// one directory and a directory made inside it are one clone domain,
/// and two real filesystems are two.
/// Why:
///   This is the only control that exercises the reading itself,
/// the device of the mount and the mount source the
///  table reports;
/// on a ZFS host it reads a real dataset,
/// the container gate reads the device rule,
/// and `/proc` is a mount of
///  its own in every container and on every host,
/// so this control never needs another dataset to compare against.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(cloneDomainRefusal(sideOf(root), sideOf(inside))).toBe(undefined);
/// expect(cloneDomainRefusal(sideOf(root), sideOf('/proc'))).not.toBe(undefined);
/// ```
#[test]
fn the_read_facts_make_one_filesystem_one_clone_domain_and_two_filesystems_two() {
    let root: PathBuf = fixture("clone-domain-read");
    let inside: PathBuf = root.join("topic");
    std::fs::create_dir(inside.as_path()).expect("a directory in the fixture root");
    let source: CloneSide = clone_side_of(root.as_path());
    let target: CloneSide = clone_side_of(inside.as_path());
    assert_eq!(clone_domain_refusal(&source, &target), None, "one directory holds both paths");
    if source.kind == FilesystemKind::Zfs {
        assert_eq!(source.mount_source, target.mount_source, "a directory made in a ZFS mount is in its dataset");
        assert!(source.mount_source.as_deref().is_some_and(|name: &str| { return !name.is_empty() }), "the mount source names the dataset");
        assert!(may_clone(root.as_path()), "a ZFS path is one the clone rung tries, so the dataset proof can refuse it");
    }
    let elsewhere: CloneSide = clone_side_of(Path::new("/proc"));
    let reason: String = clone_domain_refusal(&source, &elsewhere).expect("procfs is another filesystem than any fixture root");
    assert!(reason.contains("/proc") && reason.contains("different filesystems"), "the refusal names procfs and the cause: {reason}");
    remove(root.as_path());
}

/// What:
///  A device that agrees is not the dataset either:
/// two different dataset names are two datasets,
/// whatever
///  their devices say.
/// Why:
///   The name is what says which dataset a mount is,
/// and the device pins that name to one mount,
/// so a pair that
///  agrees on the device alone is refused rather than trusted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(cloneDomainRefusal(side(a, checkout, device), side(b, clone, device))).not.toBe(undefined);
/// ```
#[test]
fn zfs_two_dataset_names_on_one_device_are_refused() {
    let first: CloneSide = zfs_side("/home/user/Monochromatic", CHECKOUT_DATASET, CHECKOUT_DEVICE);
    let second: CloneSide = zfs_side("/home/user/Monochromatic/.claude/worktrees/topic", CLONE_DATASET, CHECKOUT_DEVICE);
    let reason: String = clone_domain_refusal(&first, &second).expect("two dataset names are two datasets");
    assert!(reason.contains(CHECKOUT_DATASET) && reason.contains(CLONE_DATASET), "{reason}");
}
