//! Controls for the in-process mount UUID:
//!  the pure decisions on literal mountinfo lines,
//!  and a differential run against the real `findmnt` as the oracle.

use super::super::filesystem_identity::{FsIdSource, create_fs_id, filesystem_identity};
use super::*;

/// Lines copied from a real host:
/// an ostree root,
/// a Btrfs subvolume mount,
/// a mount point with an escaped backslash,
/// two mounts stacked on one mount point,
/// and a pseudo filesystem.
const HOST_MOUNTINFO: &str = "\
53 1 0:42 / / ro,relatime shared:1 - overlay composefs ro,seclabel\n\
45 53 0:38 /var /var rw,relatime shared:159 - btrfs /dev/dm-1 rw,seclabel,subvolid=258,subvol=/var\n\
72 45 0:38 /home /var/home rw,relatime shared:164 - btrfs /dev/dm-1 rw,seclabel,subvolid=257,subvol=/home\n\
63 61 0:36 / /run/credentials/systemd-cryptsetup@luks\\134x2d4303.service rw - tmpfs none ro,seclabel\n\
27 60 0:34 / /proc/sys/fs/binfmt_misc rw,relatime shared:18 - autofs systemd-1 rw,fd=41\n\
226 27 0:62 / /proc/sys/fs/binfmt_misc rw,nosuid shared:230 - binfmt_misc binfmt_misc rw\n\
60 53 0:25 / /proc rw,nosuid,nodev,noexec,relatime shared:16 - proc proc rw\n\
84 53 259:3 / /boot rw,relatime shared:218 - ext4 /dev/nvme0n1p5 rw,seclabel\n\
120 84 259:1 / /boot/efi rw,relatime - vfat /dev/nvme0n1p1 rw,fmask=0077\n\
98 53 0:70 / /mnt/a\\040b rw,relatime master:3 shared:9 - fuse.example example rw\n";

/// Octal escapes decode as libmount's `unmangle_to_buffer` decodes them.
#[test]
fn octal_escapes_decode_like_libmount() {
    assert_eq!(unmangle_octal(b"a\\040b"), b"a b");
    assert_eq!(unmangle_octal(b"\\134x2d"), b"\\x2d");
    assert_eq!(unmangle_octal(b"\\011\\012"), b"\t\n");
    // The C code stores the value in a `char`.
    assert_eq!(unmangle_octal(b"\\777"), vec![0xff]);
    // Anything but a backslash and three octal digits stays.
    assert_eq!(unmangle_octal(b"\\08x"), b"\\08x");
    assert_eq!(unmangle_octal(b"\\04"), b"\\04");
    assert_eq!(unmangle_octal(b"\\"), b"\\");
    assert_eq!(unmangle_octal(b""), b"");
}

/// Hexadecimal escapes decode as libmount's `unhexmangle_to_buffer` decodes them.
#[test]
fn hexadecimal_escapes_decode_like_libmount() {
    assert_eq!(unmangle_hex(b"a\\x2db"), b"a-b");
    assert_eq!(unmangle_hex(b"\\x2D\\x2d"), b"--");
    assert_eq!(unmangle_hex(b"\\x20"), b" ");
    assert_eq!(unmangle_hex(b"\\xZZ"), b"\\xZZ");
    assert_eq!(unmangle_hex(b"\\x2"), b"\\x2");
    assert_eq!(unmangle_hex(b"\\040"), b"\\040");
    assert_eq!(unmangle_hex(b""), b"");
}

/// A real mountinfo parses to the fields that matter,
/// and one unreadable line spoils the table.
#[test]
fn mountinfo_lines_parse_with_escapes_and_optional_fields() {
    let entries: Vec<MountEntry> = parse_mountinfo(HOST_MOUNTINFO.as_bytes()).expect("a table");
    assert_eq!(entries.len(), 10);
    assert_eq!(entries[0].id, 53);
    assert_eq!(entries[0].mount_point, b"/");
    assert_eq!(entries[0].source, b"composefs");
    assert_eq!(entries[3].mount_point, b"/run/credentials/systemd-cryptsetup@luks\\x2d4303.service");
    // Two optional fields (`master:3 shared:9`) precede the separator.
    assert_eq!(entries[9].mount_point, b"/mnt/a b");
    assert_eq!(entries[9].source, b"example");
    assert_eq!(parse_mountinfo(b""), Some(Vec::new()));
    assert_eq!(parse_mountinfo(b"53 1 0:42 / / ro - overlay\n"), None);
    assert_eq!(parse_mountinfo(b"x 1 0:42 / / ro - overlay a b\n"), None);
    assert_eq!(parse_mountinfo(b"53 1 0:42 / / ro shared:1 overlay a b\n"), None);
    assert_eq!(parse_mountinfo(b"53 1 0:42 / /\n"), None);
}

/// The walk stops at the longest mount point on a component boundary,
/// and returns every entry stacked there.
#[test]
fn the_longest_mount_point_wins_on_component_boundaries() {
    let entries: Vec<MountEntry> = parse_mountinfo(HOST_MOUNTINFO.as_bytes()).expect("a table");
    let ids = |path: &[u8]| -> Vec<u64> {
        return entries_at_longest_mount_point(entries.as_slice(), path)
            .iter()
            .map(|entry: &&MountEntry| { return entry.id; })
            .collect();
    };
    assert_eq!(ids(b"/var/home/user/repo/.git"), vec![72]);
    assert_eq!(ids(b"/var/home"), vec![72]);
    assert_eq!(ids(b"/var/homework"), vec![45]);
    assert_eq!(ids(b"/var"), vec![45]);
    assert_eq!(ids(b"/"), vec![53]);
    assert_eq!(ids(b"/usr/bin"), vec![53]);
    assert_eq!(ids(b"/proc/1/status"), vec![60]);
    // Stacked mounts: both lines, in file order.
    assert_eq!(ids(b"/proc/sys/fs/binfmt_misc/x"), vec![27, 226]);
    assert_eq!(ids(b"/mnt/a b/c"), vec![98]);
    assert_eq!(ids(b"/mnt/a"), vec![53]);
    assert_eq!(entries_at_longest_mount_point(&[], b"/a"), Vec::<&MountEntry>::new());
}

/// A udev record gives its one `ID_FS_UUID_ENC`,
/// decoded,
/// and nothing that `findmnt` might print differently.
#[test]
fn udev_records_give_one_plain_uuid() {
    let plain: &[u8] = b"S:x\nE:ID_FS_UUID=1bb3d23e-ec16\nE:ID_FS_UUID_ENC=1bb3d23e-ec16\nE:ID_FS_TYPE=btrfs\n";
    assert_eq!(udev_uuid_of_record(plain).as_deref(), Some("1bb3d23e-ec16"));
    assert_eq!(
        udev_uuid_of_record(b"E:ID_FS_UUID_ENC=44C8-C6BA\n").as_deref(),
        Some("44C8-C6BA")
    );
    assert_eq!(
        udev_uuid_of_record(b"E:ID_FS_UUID_ENC=ab\\x2dcd\n").as_deref(),
        Some("ab-cd")
    );
    // No key,
    // only the unencoded key,
    // two keys,
    // an empty value,
    // a space,
    // a non-ASCII byte,
    // an underscore and an overlong value all leave the answer to `findmnt`.
    assert_eq!(udev_uuid_of_record(b"E:ID_FS_TYPE=btrfs\n"), None);
    assert_eq!(udev_uuid_of_record(b"E:ID_FS_UUID=abc\n"), None);
    assert_eq!(udev_uuid_of_record(b"E:ID_FS_UUID_ENC=a\nE:ID_FS_UUID_ENC=b\n"), None);
    assert_eq!(udev_uuid_of_record(b"E:ID_FS_UUID_ENC=\n"), None);
    assert_eq!(udev_uuid_of_record(b"E:ID_FS_UUID_ENC=a\\x20b\n"), None);
    assert_eq!(udev_uuid_of_record(b"E:ID_FS_UUID_ENC=a\\xc3\\xa9\n"), None);
    assert_eq!(udev_uuid_of_record(b"E:ID_FS_UUID_ENC=a_b\n"), None);
    assert_eq!(udev_uuid_of_record(b"E:ID_FS_UUID_ENC=a\\x00b\n"), None);
    let overlong: String = format!("E:ID_FS_UUID_ENC={}\n", "a".repeat(513));
    assert_eq!(udev_uuid_of_record(overlong.as_bytes()), None);
}

/// A pseudo source is proven empty only when no lookup `findmnt` makes could find something.
#[test]
fn pseudo_sources_need_no_lookup_to_succeed() {
    assert!(pseudo_source_has_no_uuid(b"tmpfs"));
    assert!(pseudo_source_has_no_uuid(b"composefs"));
    assert!(!pseudo_source_has_no_uuid(b""));
    assert!(!pseudo_source_has_no_uuid(b"server:/export"));
    assert!(!pseudo_source_has_no_uuid(b"UUID=1234"));
    assert!(!pseudo_source_has_no_uuid(b"."));
    assert!(!pseudo_source_has_no_uuid(b".."));
    // A name that exists in the working directory is a file `libblkid` would open by that relative name.
    let first_entry_here: Option<std::ffi::OsString> = std::fs::read_dir(".")
        .ok()
        .and_then(|mut entries| { return entries.next(); })
        .and_then(|entry| { return entry.ok(); })
        .map(|entry| { return entry.file_name(); });
    match first_entry_here {
        Some(name) => {
            use std::os::unix::ffi::OsStrExt;
            assert!(!pseudo_source_has_no_uuid(name.as_bytes()));
        }
        None => println!("skipped: the working directory is empty"),
    }
    // A name in sysfs is a block device libudev finds.
    let first_block_device: Option<std::ffi::OsString> = std::fs::read_dir("/sys/class/block")
        .ok()
        .and_then(|mut entries| { return entries.next(); })
        .and_then(|entry| { return entry.ok(); })
        .map(|entry| { return entry.file_name(); });
    match first_block_device {
        Some(name) => {
            use std::os::unix::ffi::OsStrExt;
            assert!(!pseudo_source_has_no_uuid(name.as_bytes()));
        }
        None => println!("skipped: /sys/class/block lists no block device"),
    }
}

/// The pure decision gives up on anything it cannot prove.
#[test]
fn the_decision_gives_up_on_unproven_shapes() {
    let table: &[u8] = HOST_MOUNTINFO.as_bytes();
    // A mount ID the kernel does not report for that mount point.
    assert_eq!(mount_uuid_from_table(table, b"/var/home/user", 45), MountUuid::Unknown);
    // Two stacked mounts.
    assert_eq!(mount_uuid_from_table(table, b"/proc/sys/fs/binfmt_misc", 226), MountUuid::Unknown);
    // A pseudo source with the matching ID.
    assert_eq!(mount_uuid_from_table(table, b"/proc/1", 60), MountUuid::Absent);
    assert_eq!(mount_uuid_from_table(table, b"/usr", 53), MountUuid::Absent);
    // A fuse source with a name that is no path.
    assert_eq!(mount_uuid_from_table(table, b"/mnt/a b", 98), MountUuid::Absent);
    // Paths that are not canonical.
    assert_eq!(mount_uuid_from_table(table, b"proc", 60), MountUuid::Unknown);
    assert_eq!(mount_uuid_from_table(table, b"/proc/", 60), MountUuid::Unknown);
    assert_eq!(mount_uuid_from_table(table, b"/proc//1", 60), MountUuid::Unknown);
    assert_eq!(mount_uuid_from_table(table, b"", 60), MountUuid::Unknown);
    // An unreadable table.
    assert_eq!(mount_uuid_from_table(b"garbage\n", b"/proc", 60), MountUuid::Unknown);
    // A device that does not exist here has no record to read.
    let missing_device: &[u8] = b"8 1 0:6 / /m rw - ext4 /dev/no-such-device-s2 rw\n";
    assert_eq!(mount_uuid_from_table(missing_device, b"/m/x", 8), MountUuid::Unknown);
    // A source that is a path but not a device node.
    let mounted_image: &[u8] = b"7 1 0:5 / /m rw - ext4 /var/images/disk.img rw\n";
    assert_eq!(mount_uuid_from_table(mounted_image, b"/m", 7), MountUuid::Unknown);
}

/// What `findmnt` prints for the UUID column of a canonical path,
/// or nothing when it cannot be started.
fn findmnt_uuid_output(path: &Path) -> Option<std::process::Output> {
    return std::process::Command::new("findmnt")
        .arg("--target")
        .arg(path)
        .arg("--output=UUID")
        .arg("--noheadings")
        .stdin(std::process::Stdio::null())
        .output()
        .ok();
}

/// The paths the differential test visits:
/// well-known mount points,
/// the package directory,
/// the temporary directory,
/// and any colon-separated list in `FS_IDENTITY_EXTRA_PATHS` (bind mounts,
/// overlays and the like that a script sets up).
fn differential_paths() -> Vec<std::path::PathBuf> {
    let mut paths: Vec<std::path::PathBuf> = [
        "/",
        "/tmp",
        "/proc",
        "/dev",
        "/run",
        "/sys",
        "/usr",
        "/etc",
        "/var",
        "/var/home",
        "/boot",
        "/boot/efi",
        "/proc/sys/fs/binfmt_misc",
    ]
    .iter()
    .map(std::path::PathBuf::from)
    .collect();
    paths.push(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    paths.push(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"));
    paths.push(std::env::temp_dir());
    if let Ok(extra) = std::env::var("FS_IDENTITY_EXTRA_PATHS") {
        for item in extra.split(':').filter(|item: &&str| { return !item.is_empty(); }) {
            paths.push(std::path::PathBuf::from(item));
        }
    }
    return paths;
}

/// Wherever the in-process answer is not `Unknown`,
/// it equals the bytes the real `findmnt` prints.
#[test]
fn in_process_uuids_equal_what_findmnt_prints() {
    if findmnt_uuid_output(Path::new("/")).is_none() || !Path::new(MOUNTINFO_PATH).exists() {
        println!("skipped: findmnt or {MOUNTINFO_PATH} is absent");
        return;
    }
    let mut answered: usize = 0;
    let mut text_answers: usize = 0;
    for path in differential_paths() {
        let Ok(canonical) = std::fs::canonicalize(path.as_path()) else {
            println!("skipped path: {} does not exist", path.display());
            continue;
        };
        let verdict: MountUuid = mount_uuid(canonical.as_path());
        let Some(oracle) = findmnt_uuid_output(canonical.as_path()) else {
            continue;
        };
        let printed: String = String::from_utf8_lossy(oracle.stdout.as_slice()).into_owned();
        match &verdict {
            MountUuid::Text(text) => {
                text_answers += 1;
                answered += 1;
                assert_eq!(printed, format!("{text}\n"), "uuid text for {}", canonical.display());
            }
            MountUuid::Absent => {
                answered += 1;
                assert_eq!(printed, "\n", "empty line for {}", canonical.display());
            }
            MountUuid::Unknown => {
                println!("unknown (findmnt spawns): {} -> {printed:?}", canonical.display());
            }
        }
        if verdict != MountUuid::Unknown {
            println!("{} -> {verdict:?} equals {printed:?}", canonical.display());
        }
    }
    // `/proc` is a pseudo filesystem on every Linux host that has `/proc/self/mountinfo`.
    assert_eq!(mount_uuid(Path::new("/proc")), MountUuid::Absent);
    assert!(answered > 0, "the differential run answered nothing in-process");
    println!("answered in-process: {answered}, with a UUID text: {text_answers}");
}

/// The recorded identity of a path equals the one the old command-only code computed.
#[test]
fn identities_equal_the_command_only_resolution() {
    if findmnt_uuid_output(Path::new("/")).is_none() {
        println!("skipped: findmnt is absent");
        return;
    }
    for path in differential_paths() {
        let Ok(canonical) = std::fs::canonicalize(path.as_path()) else {
            continue;
        };
        let oracle_uuid = findmnt_uuid_output(canonical.as_path()).filter(|output| { return output.status.success(); });
        let expected: Option<String> = match oracle_uuid
            .and_then(|output| {
                return create_fs_id(FsIdSource::FsUuid, String::from_utf8_lossy(output.stdout.as_slice()).as_ref());
            }) {
            Some(identity) => Some(identity),
            None => std::process::Command::new("stat")
                .arg("--file-system")
                .arg("--format=%i")
                .arg(canonical.as_path())
                .output()
                .ok()
                .filter(|output| { return output.status.success(); })
                .and_then(|output| {
                    return create_fs_id(FsIdSource::FFsid, String::from_utf8_lossy(output.stdout.as_slice()).as_ref());
                }),
        };
        assert_eq!(
            filesystem_identity(path.as_path()),
            expected,
            "identity of {}",
            path.display()
        );
    }
}
