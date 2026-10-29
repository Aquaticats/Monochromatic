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
    let directory: PathBuf = std::env::temp_dir();
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
