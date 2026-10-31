//! Controls for filesystem-identity parsing and the Linux resolver,
//!  checked against the
//! incumbent's `package/fs-id/core` parser tests and real `findmnt`/`stat` output.

use super::*;

/// Payloads are trimmed,
///  lowercased and limited to safe ASCII.
#[test]
fn payloads_normalize_as_the_incumbent_does() {
    assert_eq!(
        normalize_identity_payload(" 1A2B-3C4D\n").as_deref(),
        Some("1a2b-3c4d")
    );
    assert_eq!(
        normalize_identity_payload("abc.def").as_deref(),
        Some("abc.def")
    );
    assert_eq!(normalize_identity_payload(""), None);
    assert_eq!(normalize_identity_payload(" \n"), None);
    assert_eq!(normalize_identity_payload("-"), None);
    assert_eq!(normalize_identity_payload("a b"), None);
    assert_eq!(normalize_identity_payload("a_b"), None);
    assert_eq!(normalize_identity_payload("a\u{e9}"), None);
    // `.trim()` removes the no-break space; it never reaches the safe-character check.
    assert_eq!(
        normalize_identity_payload("\u{a0}ab\u{feff}").as_deref(),
        Some("ab")
    );
    let longest: String = "a".repeat(512);
    assert_eq!(
        normalize_identity_payload(longest.as_str()),
        Some(longest.clone())
    );
    let too_long: String = "a".repeat(513);
    assert_eq!(normalize_identity_payload(too_long.as_str()), None);
    // `toLowerCase` of the Kelvin sign is `k`, which is safe.
    assert_eq!(normalize_identity_payload("\u{212a}").as_deref(), Some("k"));
}

/// Identities carry their source prefix.
#[test]
fn identities_carry_their_source() {
    assert_eq!(
        create_fs_id(FsIdSource::FsUuid, "ABCD").as_deref(),
        Some("fs-uuid_abcd")
    );
    assert_eq!(
        create_fs_id(FsIdSource::VolumeUuid, "x").as_deref(),
        Some("volume-uuid_x")
    );
    assert_eq!(
        create_fs_id(FsIdSource::VolumeSerial, "x").as_deref(),
        Some("volume-serial_x")
    );
    assert_eq!(
        create_fs_id(FsIdSource::FFsid, "x").as_deref(),
        Some("f-fsid_x")
    );
    assert_eq!(
        create_fs_id(FsIdSource::DeviceNumber, "x").as_deref(),
        Some("device-number_x")
    );
    assert_eq!(create_fs_id(FsIdSource::FsUuid, ""), None);
}

/// `df -P` output yields the last safe `/dev/` row.
#[test]
fn df_rows_yield_the_device() {
    let output: &str = "Filesystem 1024-blocks Used Available Capacity Mounted on\n\
                        /dev/disk3s1 1 1 0 100% /\n";
    assert_eq!(parse_df_device(output).as_deref(), Some("/dev/disk3s1"));
    assert_eq!(parse_df_device("map auto_home 0 0 0 100% /home\n"), None);
    assert_eq!(parse_df_device("/dev/bad$name 1 1 0 1% /\n"), None);
    assert_eq!(
        parse_df_device("/dev/first\t1\n\n  /dev/second 1 1\n").as_deref(),
        Some("/dev/second")
    );
    // A safe row before an unsafe one still wins, as the reverse scan skips unsafe rows.
    assert_eq!(
        parse_df_device("/dev/good 1\n/dev/b@d 1\n").as_deref(),
        Some("/dev/good")
    );
    assert_eq!(parse_df_device("/dev/whole").as_deref(), Some("/dev/whole"));
    assert_eq!(parse_df_device(""), None);
}

/// The plist's `VolumeUUID` string is normalized;
///  anything missing is nothing.
#[test]
fn diskutil_plists_yield_the_volume_uuid() {
    assert_eq!(
        parse_diskutil_volume_uuid("<key>VolumeUUID</key>\n\t<string>ABCD-1234</string>")
            .as_deref(),
        Some("abcd-1234")
    );
    assert_eq!(
        parse_diskutil_volume_uuid("<key>Other</key><string>x</string>"),
        None
    );
    assert_eq!(parse_diskutil_volume_uuid("<key>VolumeUUID</key>"), None);
    assert_eq!(
        parse_diskutil_volume_uuid("<key>VolumeUUID</key><string>x"),
        None
    );
    assert_eq!(
        parse_diskutil_volume_uuid(
            "<string>early</string><key>VolumeUUID</key><string>late</string>"
        )
        .as_deref(),
        Some("late")
    );
    assert_eq!(
        parse_diskutil_volume_uuid("<key>VolumeUUID</key><string></string>"),
        None
    );
}

/// Drive roots need a letter,
///  a colon and a separator.
#[test]
fn windows_paths_need_a_drive_root() {
    assert_eq!(windows_drive_root("c:\\repo").as_deref(), Some("C:\\"));
    assert_eq!(windows_drive_root("D:/repo").as_deref(), Some("D:\\"));
    assert_eq!(windows_drive_root("1:\\repo"), None);
    assert_eq!(windows_drive_root("c\\repo"), None);
    assert_eq!(windows_drive_root("c:repo"), None);
    assert_eq!(windows_drive_root("c:"), None);
    assert_eq!(windows_drive_root("\u{e9}:\\"), None);
    assert_eq!(windows_drive_root(""), None);
}

/// The Linux resolver answers with the spelling `findmnt` or `stat` gives this directory.
#[cfg(target_os = "linux")]
#[test]
fn linux_identities_match_the_platform_commands() {
    let directory: PathBuf = crate::test_support::fixture_root();
    let found: String = filesystem_identity(directory.as_path()).expect("an identity");
    let canonical: PathBuf = std::fs::canonicalize(&directory).expect("canonical");
    let uuid: std::process::Output = std::process::Command::new("findmnt")
        .arg("--target")
        .arg(&canonical)
        .arg("--output=UUID")
        .arg("--noheadings")
        .output()
        .expect("findmnt");
    let expected: String = match create_fs_id(
        FsIdSource::FsUuid,
        String::from_utf8_lossy(uuid.stdout.as_slice()).as_ref(),
    ) {
        Some(identity) if uuid.status.success() => identity,
        _ => {
            let fsid: std::process::Output = std::process::Command::new("stat")
                .arg("--file-system")
                .arg("--format=%i")
                .arg(&canonical)
                .output()
                .expect("stat");
            create_fs_id(
                FsIdSource::FFsid,
                String::from_utf8_lossy(fsid.stdout.as_slice()).as_ref(),
            )
            .expect("f_fsid")
        }
    };
    assert_eq!(found, expected);
    assert_eq!(
        filesystem_identity(directory.join("missing-entry").as_path()),
        None
    );
}

/// A failing command is no answer,
///  and `stat` is the fallback.
#[cfg(target_os = "linux")]
#[test]
fn failing_commands_are_no_answer() {
    assert_eq!(command_output("/nonexistent/command", &[]), None);
    assert_eq!(command_output("false", &[]), None);
    assert_eq!(
        command_output("printf", &[std::ffi::OsStr::new("x")]).as_deref(),
        Some("x")
    );
    assert_eq!(linux_fs_id(Path::new("/nonexistent/path")), None);
}

#[cfg(target_os = "linux")]
thread_local! {
    /// The commands the replaced mechanism was asked to start, in order.
    static STARTED_COMMANDS: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// A mechanism that starts nothing:
/// it records the command and answers an empty line, as `findmnt` does for a filesystem without a UUID.
#[cfg(target_os = "linux")]
fn recording_runner(command: &str, _arguments: &[&std::ffi::OsStr]) -> Option<String> {
    STARTED_COMMANDS.with(|started: &std::cell::RefCell<Vec<String>>| {
        return started.borrow_mut().push(String::from(command));
    });
    return Some(String::from("\n"));
}

/// The degraded identity of a path as the kernel reports it, spelled as `stat` spells it.
#[cfg(target_os = "linux")]
fn in_process_f_fsid(path: &Path) -> String {
    return format!("f-fsid_{}", super::super::filesystem_identity_fsid::f_fsid_text(path).expect("a text"));
}

/// The commands recorded since the last call,
/// cleared.
#[cfg(target_os = "linux")]
fn take_started_commands() -> Vec<String> {
    return STARTED_COMMANDS.with(|started: &std::cell::RefCell<Vec<String>>| {
        return std::mem::take(&mut *started.borrow_mut());
    });
}

/// A pseudo filesystem needs no `findmnt` and, with the `f_fsid` read in-process, no command at all.
#[cfg(target_os = "linux")]
#[test]
fn a_pseudo_filesystem_starts_nothing() {
    if !Path::new("/proc/self/mountinfo").exists() {
        println!("skipped: /proc/self/mountinfo is absent");
        return;
    }
    take_started_commands();
    assert_eq!(
        linux_fs_id_with(Path::new("/proc"), recording_runner),
        Some(in_process_f_fsid(Path::new("/proc")))
    );
    assert_eq!(
        take_started_commands(),
        Vec::<String>::new(),
        "in-process verdict for /proc: {:?}",
        super::super::filesystem_identity_mount::mount_uuid(Path::new("/proc"))
    );
}

/// A host that refuses every process start still resolves the degraded identity of a path the mount table cannot
/// name by UUID: the answer comes from the kernel, so a release of a lock this process holds does not depend on a start.
#[cfg(target_os = "linux")]
#[test]
fn the_degraded_identity_needs_no_process_start() {
    fn refusing_runner(_command: &str, _arguments: &[&std::ffi::OsStr]) -> Option<String> {
        return None;
    }
    assert_eq!(
        linux_fs_id_with(Path::new("/proc"), refusing_runner),
        Some(in_process_f_fsid(Path::new("/proc")))
    );
}

/// A device-backed path with a udev record starts no command at all.
#[cfg(target_os = "linux")]
#[test]
fn a_device_with_a_udev_record_starts_nothing() {
    let Ok(directory) = std::fs::canonicalize(env!("CARGO_MANIFEST_DIR")) else {
        println!("skipped: the package directory is absent here");
        return;
    };
    let super::super::filesystem_identity_mount::MountUuid::Text(text) =
        super::super::filesystem_identity_mount::mount_uuid(directory.as_path())
    else {
        println!("skipped: the package directory is not on a device with a udev record");
        return;
    };
    take_started_commands();
    assert_eq!(
        linux_fs_id_with(directory.as_path(), recording_runner),
        Some(format!("fs-uuid_{}", text.to_lowercase()))
    );
    assert_eq!(take_started_commands(), Vec::<String>::new());
}

/// A shape the in-process lookup cannot prove still starts `findmnt` first.
#[cfg(target_os = "linux")]
#[test]
fn an_unprovable_path_still_starts_findmnt() {
    let stacked: &Path = Path::new("/proc/sys/fs/binfmt_misc");
    if super::super::filesystem_identity_mount::mount_uuid(stacked)
        != super::super::filesystem_identity_mount::MountUuid::Unknown
    {
        println!("skipped: {} is not an unprovable mount here", stacked.display());
        return;
    }
    take_started_commands();
    assert_eq!(
        linux_fs_id_with(stacked, recording_runner),
        Some(in_process_f_fsid(stacked))
    );
    assert_eq!(take_started_commands(), vec![String::from("findmnt")]);
}
