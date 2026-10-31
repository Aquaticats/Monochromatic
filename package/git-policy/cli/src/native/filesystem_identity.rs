//! What:
//!  The filesystem identity recorded with every real `index.lock` a transaction creates,
//!       spelled exactly as the incumbent's `@monochromatic-dev/fs-id-core` spells it.
//! Why:
//!  Recovery removes a real `index.lock` only when its device,
//!  inode and filesystem
//!      identity all match the journal;
//!  the incumbent compares the recorded `fsId` with the one
//!      it resolves now,
//!  so both wrappers must resolve the same text for the same filesystem
//!      (`package/fs-id/core/src/platform-resolvers.ts`,
//!  `parsers.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { value } = await resolveFsId({ path: lockPath, emitDiagnostics: false });
//! ```

/// Trimming with the incumbent's `.trim()` semantics.
use super::js_text::trim_javascript;
/// Starts that ride out a short shortage of processes.
use super::start_retry::RetryStart;
/// The in-process answer for the `f_fsid` GNU `stat` would print.
#[cfg(target_os = "linux")]
use super::filesystem_identity_fsid::f_fsid_text;
/// The in-process answer for the UUID `findmnt` would print.
#[cfg(target_os = "linux")]
use super::filesystem_identity_mount::{MountUuid, mount_uuid};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// Longest accepted platform payload.
const MAX_PAYLOAD_LENGTH: usize = 512;

/// Characters a normalized payload may hold.
const SAFE_PAYLOAD_CHARACTERS: &str = "abcdefghijklmnopqrstuvwxyz0123456789-.";

/// Characters a macOS device-node path may hold.
const SAFE_DEVICE_CHARACTERS: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789/_-.";

/// What:
///  The mechanism that supplied an identity,
///  which names its prefix.
/// Why:
///   `SOURCE_PREFIXES` in the incumbent:
///  stable sources and degraded fallbacks are told
///       apart in the text itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FsIdSource = 'fs-uuid' | 'volume-uuid' | 'volume-serial' | 'f-fsid' | 'device-number';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FsIdSource {
    /// A Linux filesystem UUID from `findmnt`.
    FsUuid,
    /// A macOS Volume UUID from `diskutil`.
    VolumeUuid,
    /// A Windows volume serial number.
    VolumeSerial,
    /// A Linux `f_fsid`, spelled as `stat --file-system --format=%i` spells it and read in-process.
    FFsid,
    /// A device number.
    DeviceNumber,
}

/// What:
///  The prefix of one source.
/// Why:
///   The identity text is `<prefix><payload>`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// SOURCE_PREFIXES[source]
/// ```
pub fn source_prefix(source: FsIdSource) -> &'static str {
    match source {
        FsIdSource::FsUuid => return "fs-uuid_",
        FsIdSource::VolumeUuid => return "volume-uuid_",
        FsIdSource::VolumeSerial => return "volume-serial_",
        FsIdSource::FFsid => return "f-fsid_",
        FsIdSource::DeviceNumber => return "device-number_",
    }
}

/// What:
///  Normalize one command-supplied payload:
///  trim,
///  lowercase,
///  then accept only non-empty
///       safe ASCII up to 512 characters that is not a lone `-`.
/// Why:
///   `normalizeIdentityPayload`;
///  anything else is no identity.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function normalizeIdentityPayload(value: string): string; // throws when unsafe
/// ```
pub fn normalize_identity_payload(value: &str) -> Option<String> {
    let normalized: String = trim_javascript(value).to_lowercase();
    // `chars().count()` counts characters, as JavaScript's `length` counts code units; both
    // are equal for the ASCII text that alone can pass.
    let length: usize = normalized.chars().count();
    if length == 0
        || length > MAX_PAYLOAD_LENGTH
        || normalized == "-"
        || trim_javascript(normalized.as_str()) != normalized
    {
        return None;
    }
    for character in normalized.chars() {
        if !SAFE_PAYLOAD_CHARACTERS.contains(character) {
            return None;
        }
    }
    return Some(normalized);
}

/// What:
///  The identity text `<prefix><payload>` for a normalized payload.
/// Why:
///   `createFsId`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// createFsId({ source, payload })
/// ```
pub fn create_fs_id(source: FsIdSource, payload: &str) -> Option<String> {
    return Some(format!(
        "{}{}",
        source_prefix(source),
        normalize_identity_payload(payload)?
    ));
}

/// What:
///  The mounted device node in POSIX `df -P` output:
///  the first field of the last
///       non-empty row that starts with `/dev/` and holds only safe characters.
/// Why:
///   `parseDfDevice` on macOS.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseDfDevice(output: string): string; // throws when none
/// ```
pub fn parse_df_device(output: &str) -> Option<String> {
    let mut found: Option<String> = None;
    for line in output.split('\n') {
        let trimmed: &str = trim_javascript(line);
        if trimmed.is_empty() {
            continue;
        }
        let end: usize = trimmed.find([' ', '\t']).unwrap_or(trimmed.len());
        let device: &str = &trimmed[..end];
        if device.starts_with("/dev/") && device.chars().all(is_safe_device_character) {
            // A later row wins: the incumbent scans the rows in reverse and takes the first.
            found = Some(String::from(device));
        }
    }
    return found;
}

/// What:
///  Whether a character may appear in a device-node path.
/// Why:
///   A named predicate keeps the scan free of closures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// SAFE_DEVICE_CHARACTERS.includes(character)
/// ```
fn is_safe_device_character(character: char) -> bool {
    return SAFE_DEVICE_CHARACTERS.contains(character);
}

/// What:
///  The `VolumeUUID` string of a `diskutil info -plist` property list,
///  normalized.
/// Why:
///   `parseDiskutilVolumeUuid` on macOS.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseDiskutilVolumeUuid(output: string): string; // throws when absent
/// ```
pub fn parse_diskutil_volume_uuid(output: &str) -> Option<String> {
    let key: &str = "<key>VolumeUUID</key>";
    let open: &str = "<string>";
    let close: &str = "</string>";
    let key_index: usize = output.find(key)?;
    let open_index: usize = key_index + key.len() + output[key_index + key.len()..].find(open)?;
    let value_start: usize = open_index + open.len();
    let value_end: usize = value_start + output[value_start..].find(close)?;
    return normalize_identity_payload(&output[value_start..value_end]);
}

/// What:
///  The uppercase drive root `X:\` of a Windows path,
///  or nothing for a path without one.
/// Why:
///   `windowsDriveRoot`:
///  the drive letter feeds a fixed PowerShell query.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function windowsDriveRoot(path: string): string; // throws without a drive root
/// ```
pub fn windows_drive_root(path: &str) -> Option<String> {
    let mut characters: std::str::Chars<'_> = path.chars();
    let letter: char = characters.next()?.to_ascii_lowercase();
    let colon: char = characters.next()?;
    let separator: char = characters.next()?;
    if !letter.is_ascii_lowercase() || colon != ':' || (separator != '\\' && separator != '/') {
        return None;
    }
    return Some(format!("{}:\\", letter.to_ascii_uppercase()));
}

/// What:
///  Run one platform command and return its standard output as text,
///  or nothing when it
///       cannot start or exits unsuccessfully.
/// Why:
///   The incumbent's `nano-spawn` call throws on either;
///  both send it to the fallback.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { stdout } = await nanoSpawn(command, args);
/// ```
fn command_output(command: &str, arguments: &[&std::ffi::OsStr]) -> Option<String> {
    let output: std::process::Output = std::process::Command::new(command)
        .args(arguments)
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output_retrying()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    return Some(String::from_utf8_lossy(output.stdout.as_slice()).into_owned());
}

/// What:
///  The Linux identity:
///  the filesystem UUID from `findmnt`,
///  or the degraded `f_fsid`
///       the kernel reports, spelled as GNU `stat` spells it.
/// Why:
///   `resolveLinuxFsId`,
///  step for step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveLinuxFsId({ path }): Promise<FsIdResolution>;
/// ```
#[cfg(target_os = "linux")]
fn linux_fs_id(path: &Path) -> Option<String> {
    return linux_fs_id_with(path, command_output);
}

/// What:
///  The Linux identity of `linux_fs_id` with the mechanism that starts a platform command passed in.
/// Why:
///   A control that counts which commands start can only do so by replacing the mechanism.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveLinuxFsId({ path, run }): Promise<FsIdResolution>;
/// ```
#[cfg(target_os = "linux")]
fn linux_fs_id_with(
    path: &Path,
    run_command: fn(&str, &[&std::ffi::OsStr]) -> Option<String>,
) -> Option<String> {
    let target: &std::ffi::OsStr = path.as_os_str();
    // The mount table and the udev record answer most paths without starting `findmnt`;
    // a shape they cannot answer exactly is `Unknown` and still spawns it.
    let in_process: MountUuid = mount_uuid(path);
    match in_process {
        MountUuid::Text(text) => {
            if let Some(identity) = create_fs_id(FsIdSource::FsUuid, text.as_str()) {
                return Some(identity);
            }
        }
        MountUuid::Absent => {}
        MountUuid::Unknown => {
            if let Some(output) = run_command(
                "findmnt",
                &[
                    std::ffi::OsStr::new("--target"),
                    target,
                    std::ffi::OsStr::new("--output=UUID"),
                    std::ffi::OsStr::new("--noheadings"),
                ],
            ) && let Some(identity) = create_fs_id(FsIdSource::FsUuid, output.as_str())
            {
                return Some(identity);
            }
        }
    }
    // The degraded identity needs no process: the kernel answers `statfs` in-process, so releasing a lock this
    // process holds still works when the host refuses every start.
    let fallback: String = f_fsid_text(path)?;
    return create_fs_id(FsIdSource::FFsid, fallback.as_str());
}

/// What:
///  The filesystem identity of the filesystem holding `path`,
///  or nothing when no
///       mechanism answers.
/// Why:
///   The incumbent canonicalizes the path first and then asks the platform's mechanism;
///       on macOS and Windows the commands are the incumbent's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await resolveFsId({ path })).value
/// ```
pub fn filesystem_identity(path: &Path) -> Option<String> {
    let canonical: PathBuf = std::fs::canonicalize(path).ok()?;
    #[cfg(target_os = "linux")]
    {
        return linux_fs_id(canonical.as_path());
    }
    #[cfg(target_os = "macos")]
    {
        return darwin_fs_id(canonical.as_path());
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(windows)]
    {
        return windows_fs_id(canonical.as_path());
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    {
        let _ = canonical;
        return None;
    }
}

/// What:
///  The macOS identity:
///  the Volume UUID through `df -P` and `diskutil info -plist`,
///  or the
///       degraded device number from BSD `stat -f %d`.
/// Why:
///   `resolveDarwinFsId`,
///  step for step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveDarwinFsId({ path }): Promise<FsIdResolution>;
/// ```
#[cfg(target_os = "macos")]
fn darwin_fs_id(path: &Path) -> Option<String> {
    let target: &std::ffi::OsStr = path.as_os_str();
    if let Some(mounts) = command_output("df", &[std::ffi::OsStr::new("-P"), target])
        && let Some(device) = parse_df_device(mounts.as_str())
        && let Some(plist) = command_output(
            "diskutil",
            &[
                std::ffi::OsStr::new("info"),
                std::ffi::OsStr::new("-plist"),
                std::ffi::OsStr::new(device.as_str()),
            ],
        )
        && let Some(uuid) = parse_diskutil_volume_uuid(plist.as_str())
    {
        return Some(format!("{}{uuid}", source_prefix(FsIdSource::VolumeUuid)));
    }
    let number: String = command_output(
        "stat",
        &[
            std::ffi::OsStr::new("-f"),
            std::ffi::OsStr::new("%d"),
            target,
        ],
    )?;
    return create_fs_id(FsIdSource::DeviceNumber, number.as_str());
}

/// What:
///  The Windows identity:
///  the volume serial number through PowerShell.
/// Why:
///   `resolveWindowsFsId`'s preferred step;
///  its device-number fallback is not ported
///       (recorded in `doc/handover/cli-git-native-transactions.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveWindowsFsId({ path }): Promise<FsIdResolution>;
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(windows)]
fn windows_fs_id(path: &Path) -> Option<String> {
    let text: String = path.to_string_lossy().into_owned();
    if let Some(root) = windows_drive_root(text.as_str()) {
        let query: String = format!(
            "(Get-CimInstance Win32_LogicalDisk -Filter \"DeviceID='{}'\").VolumeSerialNumber",
            &root[..2]
        );
        if let Some(output) = command_output(
            "powershell.exe",
            &[
                std::ffi::OsStr::new("-NoLogo"),
                std::ffi::OsStr::new("-NoProfile"),
                std::ffi::OsStr::new("-NonInteractive"),
                std::ffi::OsStr::new("-Command"),
                std::ffi::OsStr::new(query.as_str()),
            ],
        ) && let Some(identity) = create_fs_id(FsIdSource::VolumeSerial, output.as_str())
        {
            return Some(identity);
        }
    }
    // The incumbent falls back to Node's device number, the volume serial number that only an
    // unstable standard-library interface exposes; without it no identity is recorded, and the
    // landing that needs one stops instead of recording a different spelling.
    return None;
}

/// Parser controls and the Linux resolver stay out of the release executable.
#[cfg(test)]
#[path = "filesystem_identity_tests.rs"]
mod tests;
