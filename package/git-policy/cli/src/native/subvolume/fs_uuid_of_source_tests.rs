//! Controls for the filesystem UUID of a mount's source device:
//! a link found through an alias of the device,
//! a different device,
//! and the sources that are not devices.

use super::*;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// A folder with two device stand-ins,
/// an alias of the first (the way `/dev/mapper/x` leads to `/dev/dm-0`),
/// and a by-uuid folder with one link to each device.
struct Devices {
    root: PathBuf,
    first_alias: PathBuf,
    second: PathBuf,
    by_uuid: PathBuf,
}

fn devices(name: &str) -> Devices {
    let root: PathBuf = fixture(name);
    let dev: PathBuf = root.join("dev");
    let by_uuid: PathBuf = root.join("by-uuid");
    let mapper: PathBuf = root.join("mapper");
    for directory in [&dev, &by_uuid, &mapper] {
        std::fs::create_dir_all(directory).expect("directory");
    }
    let first: PathBuf = dev.join("dm-0");
    let second: PathBuf = dev.join("dm-1");
    std::fs::write(&first, b"").expect("first device");
    std::fs::write(&second, b"").expect("second device");
    std::os::unix::fs::symlink(&first, by_uuid.join("1111-aaaa")).expect("first uuid link");
    std::os::unix::fs::symlink(&second, by_uuid.join("2222-bbbb")).expect("second uuid link");
    let first_alias: PathBuf = mapper.join("crypt");
    std::os::unix::fs::symlink(&first, &first_alias).expect("alias");
    return Devices { root, first_alias, second, by_uuid };
}

/// The UUID link is found through an alias of the device,
/// and the other device keeps its own.
#[test]
fn the_uuid_of_a_device_is_found_through_an_alias_of_it() {
    let devices: Devices = devices("uuid-of-source-alias");
    assert_eq!(filesystem_uuid_of_source(devices.first_alias.to_str().expect("text"), devices.by_uuid.as_path()), Some(String::from("1111-aaaa")));
    assert_eq!(filesystem_uuid_of_source(devices.second.to_str().expect("text"), devices.by_uuid.as_path()), Some(String::from("2222-bbbb")));
    remove(devices.root.as_path());
}

/// A source that is not a path to a device has no UUID,
/// and a relative one is never resolved against the working directory.
#[test]
fn a_source_that_is_not_a_device_path_has_no_uuid() {
    let devices: Devices = devices("uuid-of-source-none");
    assert_eq!(filesystem_uuid_of_source("tmpfs", devices.by_uuid.as_path()), None);
    assert_eq!(filesystem_uuid_of_source("overlay", devices.by_uuid.as_path()), None);
    assert_eq!(filesystem_uuid_of_source("dev/dm-0", devices.by_uuid.as_path()), None, "a relative source");
    assert_eq!(filesystem_uuid_of_source(devices.root.join("absent").to_str().expect("text"), devices.by_uuid.as_path()), None);
    remove(devices.root.as_path());
}

/// A device without a link,
/// and a folder of links that is not there,
/// give no UUID.
#[test]
fn a_device_without_a_link_or_without_a_folder_has_no_uuid() {
    let devices: Devices = devices("uuid-of-source-unlinked");
    let third: PathBuf = devices.root.join("dev/dm-2");
    std::fs::write(&third, b"").expect("third device");
    assert_eq!(filesystem_uuid_of_source(third.to_str().expect("text"), devices.by_uuid.as_path()), None);
    assert_eq!(filesystem_uuid_of_source(devices.second.to_str().expect("text"), devices.root.join("no-such-folder").as_path()), None);
    remove(devices.root.as_path());
}
