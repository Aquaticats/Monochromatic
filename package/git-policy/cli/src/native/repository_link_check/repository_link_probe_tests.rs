//! Controls for the real probe over a mount table and a folder of UUID links the test made:
//! which filesystem holds a path,
//! its UUID,
//! and which paths are mount points,
//! with no real mount.

use super::*;
use crate::test_support::{fixture, remove};

/// A root with a stand-in device for the root mount and one for a mounted drive,
/// a folder of UUID links to both,
/// and a mount table that lists both mounts.
struct World {
    root: PathBuf,
    mount_point: PathBuf,
    probe: RealLinkProbe,
}

fn world(name: &str) -> World {
    let root: PathBuf = fixture(name).canonicalize().expect("root");
    let dev: PathBuf = root.join("dev");
    let by_uuid: PathBuf = root.join("by-uuid");
    let mount_point: PathBuf = root.join("mnt");
    std::fs::create_dir_all(&dev).expect("dev");
    std::fs::create_dir_all(&by_uuid).expect("by-uuid");
    std::fs::create_dir_all(mount_point.join("folder")).expect("mount point");
    let root_device: PathBuf = dev.join("root");
    let drive_device: PathBuf = dev.join("drive");
    std::fs::write(&root_device, b"").expect("root device");
    std::fs::write(&drive_device, b"").expect("drive device");
    std::os::unix::fs::symlink(&root_device, by_uuid.join("ROOT-UUID")).expect("root link");
    std::os::unix::fs::symlink(&drive_device, by_uuid.join("DRIVE-UUID")).expect("drive link");
    let table: String = format!(
        "100 1 0:30 / {} rw - btrfs {} rw\n101 100 0:31 / {} rw - btrfs {} rw\n",
        root.display(),
        root_device.display(),
        mount_point.display(),
        drive_device.display()
    );
    return World { root, mount_point, probe: RealLinkProbe::over(table.into_bytes(), by_uuid) };
}

/// A path is on the filesystem of the longest mount that holds it,
/// and the UUID is the one the device's link names;
/// asking again gives the same answers.
#[test]
fn a_path_has_the_uuid_of_the_filesystem_that_holds_it() {
    let world: World = world("probe-uuid");
    assert_eq!(world.probe.uuid_of(world.mount_point.join("folder").as_path()), "DRIVE-UUID");
    assert_eq!(world.probe.uuid_of(world.root.join("dev").as_path()), "ROOT-UUID");
    assert_eq!(world.probe.uuid_of(world.mount_point.join("folder").as_path()), "DRIVE-UUID", "asked again");
    assert_eq!(world.probe.uuid_of(world.root.join("absent").as_path()), "", "a path that is not there has none");
    remove(world.root.as_path());
}

/// Only the mount point itself is a mount point;
/// a folder in it and a path that is not there are not.
#[test]
fn only_the_mount_point_is_a_mount_point() {
    let world: World = world("probe-mount-point");
    assert!(world.probe.is_mount_point(world.mount_point.as_path()));
    assert!(!world.probe.is_mount_point(world.mount_point.join("folder").as_path()));
    assert!(!world.probe.is_mount_point(world.root.join("absent").as_path()));
    remove(world.root.as_path());
}

/// A device with no UUID link has no UUID.
#[test]
fn a_device_without_a_uuid_link_has_none() {
    let world: World = world("probe-no-link");
    std::fs::remove_file(world.root.join("by-uuid/DRIVE-UUID")).expect("remove the drive's link");
    assert_eq!(world.probe.uuid_of(world.mount_point.join("folder").as_path()), "");
    remove(world.root.as_path());
}
