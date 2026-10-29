//! What:
//!  The Linux filesystem UUID of the filesystem holding a path,
//!  read in-process from the kernel's mount table and the udev database instead of by starting `findmnt`.
//! Why:
//!  `findmnt --target <path> --output=UUID --noheadings` is the most expensive process of a wrapped commit,
//!  and the recorded identity must stay byte-identical to what it prints;
//!  every shape this module does not reproduce exactly is reported as unknown so the caller spawns `findmnt` as before.
//!  `doc/planning/cli-git-spawn-and-sync-reduction.md` (section "Filesystem identity without `findmnt`")
//!  holds the algorithm read from util-linux and the measurements.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { stdout } = await nanoSpawn('findmnt', ['--target', path, '--output=UUID', '--noheadings']);
//! ```

/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// Where the kernel lists the mounts of this process's mount namespace,
/// the file `findmnt` reads first.
const MOUNTINFO_PATH: &str = "/proc/self/mountinfo";

/// Where udev keeps one record per block device, named `b<major>:<minor>`.
const UDEV_DATA_DIRECTORY: &str = "/run/udev/data";

/// The sysfs directories libudev looks a block device up in by name, in its order.
const SYSFS_BLOCK_DIRECTORIES: [&str; 2] = ["/sys/class/block", "/sys/bus/block/devices"];

/// The udev property `findmnt` reads for the UUID column.
const UDEV_UUID_PROPERTY_LINE: &str = "E:ID_FS_UUID_ENC=";

/// Longest UUID text accepted,
/// the same bound as the identity payload normalizer.
const MAX_UUID_LENGTH: usize = 512;

/// What:
///  What `findmnt` would print for the UUID column of a path,
///  as far as this module can prove it without starting `findmnt`.
/// Why:
///   The caller records an identity from `Text`,
///  skips `findmnt` for `Absent`,
///  and spawns `findmnt` for `Unknown`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountUuid = { kind: 'text'; text: string } | { kind: 'absent' } | { kind: 'unknown' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MountUuid {
    /// `findmnt` prints exactly this text and a newline.
    Text(String),
    /// `findmnt` prints an empty line:
    /// the mounted source is a pseudo filesystem with no UUID.
    Absent,
    /// This module cannot prove what `findmnt` prints.
    Unknown,
}

/// What:
///  One line of `/proc/self/mountinfo`,
///  with the escapes of the mount point and the source undone.
/// Why:
///   Only the mount ID,
///  the mount point and the source take part in finding the mount of a path and its device.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountEntry = { id: number; mountPoint: Uint8Array; source: Uint8Array };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MountEntry {
    /// The kernel's mount ID,
    /// the first field,
    /// which `statx` reports as `stx_mnt_id`.
    pub id: u64,
    /// The mount point as seen from this process's root,
    /// octal escapes decoded.
    pub mount_point: Vec<u8>,
    /// The mount source after the dash separator,
    /// octal escapes decoded.
    pub source: Vec<u8>,
}

/// What:
///  Decode the `\ooo` octal escapes of a mountinfo field exactly as libmount's `unmangle_to_buffer` does:
///  a backslash followed by three octal digits becomes one byte,
///  anything else stays.
/// Why:
///   The kernel writes space,
///  tab,
///  newline and backslash in mount points as `\040`,
///  `\011`,
///  `\012` and `\134`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// field.replace(/\\([0-7]{3})/g, (_, octal) => String.fromCharCode(parseInt(octal, 8) & 0xff));
/// ```
pub fn unmangle_octal(field: &[u8]) -> Vec<u8> {
    let mut decoded: Vec<u8> = Vec::with_capacity(field.len());
    let mut index: usize = 0;
    while index < field.len() {
        let is_escape: bool = field[index] == b'\\'
            && index + 3 < field.len()
            && field[index + 1..index + 4]
                .iter()
                .all(|digit: &u8| { return (b'0'..=b'7').contains(digit); });
        if is_escape {
            let value: u32 = 64 * u32::from(field[index + 1] & 7)
                + 8 * u32::from(field[index + 2] & 7)
                + u32::from(field[index + 3] & 7);
            // The C code stores the value in a `char`, so `\777` wraps to 0xff.
            decoded.push((value & 0xff) as u8);
            index += 4;
        } else {
            decoded.push(field[index]);
            index += 1;
        }
    }
    return decoded;
}

/// What:
///  Decode the `\xHH` hexadecimal escapes of a udev property exactly as libmount's `unhexmangle_to_buffer` does:
///  a backslash,
///  `x` and two hexadecimal digits become one byte,
///  anything else stays.
/// Why:
///   `ID_FS_UUID_ENC` is udev's escaped spelling of the UUID,
///  which `findmnt` decodes before printing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// value.replace(/\\x([0-9a-fA-F]{2})/g, (_, hex) => String.fromCharCode(parseInt(hex, 16)));
/// ```
pub fn unmangle_hex(value: &[u8]) -> Vec<u8> {
    let mut decoded: Vec<u8> = Vec::with_capacity(value.len());
    let mut index: usize = 0;
    while index < value.len() {
        let is_escape: bool = value[index] == b'\\'
            && index + 3 < value.len()
            && value[index + 1] == b'x'
            && value[index + 2].is_ascii_hexdigit()
            && value[index + 3].is_ascii_hexdigit();
        if is_escape {
            let high: u8 = hex_digit_value(value[index + 2]);
            let low: u8 = hex_digit_value(value[index + 3]);
            decoded.push((high << 4) | low);
            index += 4;
        } else {
            decoded.push(value[index]);
            index += 1;
        }
    }
    return decoded;
}

/// What:
///  The value of one ASCII hexadecimal digit.
/// Why:
///   `unmangle_hex` has checked the digit already.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseInt(String.fromCharCode(digit), 16)
/// ```
fn hex_digit_value(digit: u8) -> u8 {
    match digit {
        b'0'..=b'9' => return digit - b'0',
        b'a'..=b'f' => return digit - b'a' + 10,
        _ => return digit - b'A' + 10,
    }
}

/// What:
///  Parse the text of `/proc/self/mountinfo`,
///  or nothing when any line does not have the documented shape.
/// Why:
///   A line this parser cannot read might be the one `findmnt` would choose,
///  so one unreadable line makes the whole answer unknown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseMountinfo(text: Uint8Array): MountEntry[] | undefined;
/// ```
pub fn parse_mountinfo(text: &[u8]) -> Option<Vec<MountEntry>> {
    let mut entries: Vec<MountEntry> = Vec::new();
    for line in text.split(|byte: &u8| { return *byte == b'\n'; }) {
        if line.is_empty() {
            continue;
        }
        entries.push(parse_mountinfo_line(line)?);
    }
    return Some(entries);
}

/// What:
///  Parse one mountinfo line:
///  `id parent major:minor root mount-point options [optional...] - fstype source super-options`.
/// Why:
///   The fields are separated by single spaces because the kernel escapes spaces inside a field.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseMountinfoLine(line: Uint8Array): MountEntry | undefined;
/// ```
fn parse_mountinfo_line(line: &[u8]) -> Option<MountEntry> {
    let fields: Vec<&[u8]> = line.split(|byte: &u8| { return *byte == b' '; }).collect();
    let separator: usize = fields.iter().position(|field: &&[u8]| { return *field == b"-"; })?;
    // Six fixed fields precede the optional ones,
    // and the separator is followed by the file system type,
    // the source and the super options.
    if separator < 6 || fields.len() < separator + 4 {
        return None;
    }
    let id: u64 = std::str::from_utf8(fields[0]).ok()?.parse::<u64>().ok()?;
    return Some(MountEntry {
        id,
        mount_point: unmangle_octal(fields[4]),
        source: unmangle_octal(fields[separator + 2]),
    });
}

/// What:
///  Every entry whose mount point is the longest ancestor-or-self of `path` that any entry mounts on,
///  in file order.
/// Why:
///   `findmnt --target` walks the path up one component at a time until some entry's mount point equals it,
///  then prints every entry with that mount point
///       (`mnt_table_find_mountpoint` followed by `add_matching_lines`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entriesAtLongestMountPoint(entries: MountEntry[], path: Uint8Array): MountEntry[];
/// ```
pub fn entries_at_longest_mount_point<'entries>(
    entries: &'entries [MountEntry],
    path: &[u8],
) -> Vec<&'entries MountEntry> {
    let mut candidate: &[u8] = path;
    loop {
        let found: Vec<&MountEntry> = entries
            .iter()
            .filter(|entry: &&MountEntry| { return entry.mount_point.as_slice() == candidate; })
            .collect();
        if !found.is_empty() {
            return found;
        }
        match candidate.iter().rposition(|byte: &u8| { return *byte == b'/'; }) {
            Some(0) if candidate.len() > 1 => candidate = &candidate[..1],
            Some(position) if position > 0 => candidate = &candidate[..position],
            _ => return Vec::new(),
        }
    }
}

/// What:
///  The UUID `findmnt` prints for a source that is not a device path,
///  or nothing when the source needs a closer look.
/// Why:
///   `findmnt` looks such a name up as a sysfs block device in both libudev directories
///  and lets `libblkid` open it as a relative path;
///  only a name that exists in none of them provably has no UUID.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pseudoSourceHasNoUuid(source: string): boolean;
/// ```
pub fn pseudo_source_has_no_uuid(source: &[u8]) -> bool {
    use std::os::unix::ffi::OsStrExt;
    if source.is_empty()
        || source.contains(&b'/')
        || source.contains(&b'=')
        || source.contains(&0)
        || source == b"."
        || source == b".."
    {
        return false;
    }
    let name: &std::ffi::OsStr = std::ffi::OsStr::from_bytes(source);
    for directory in SYSFS_BLOCK_DIRECTORIES {
        if !is_surely_missing(Path::new(directory).join(name).as_path()) {
            return false;
        }
    }
    return is_surely_missing(Path::new(name));
}

/// What:
///  Whether a path certainly does not exist:
///  the lookup reported "not found" and nothing else.
/// Why:
///   A permission error or any other failure leaves the question open.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { await lstat(path); return false; } catch (e) { return e.code === 'ENOENT'; }
/// ```
fn is_surely_missing(path: &Path) -> bool {
    match std::fs::symlink_metadata(path) {
        Ok(_) => return false,
        Err(error) => return error.kind() == std::io::ErrorKind::NotFound,
    }
}

/// What:
///  The `ID_FS_UUID_ENC` value of a udev database record,
///  decoded,
///  or nothing unless the record holds exactly one such line whose value `findmnt` prints unchanged.
/// Why:
///   The accepted text is limited to ASCII letters,
///  digits,
///  dots and dashes,
///  which libsmartcols writes verbatim;
///  anything else may be re-encoded when printed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function udevUuid(record: string): string | undefined;
/// ```
pub fn udev_uuid_of_record(record: &[u8]) -> Option<String> {
    let prefix: &[u8] = UDEV_UUID_PROPERTY_LINE.as_bytes();
    let mut values = record
        .split(|byte: &u8| { return *byte == b'\n'; })
        .filter(|line: &&[u8]| { return line.starts_with(prefix); });
    let value: &[u8] = &values.next()?[prefix.len()..];
    if values.next().is_some() {
        return None;
    }
    let decoded: Vec<u8> = unmangle_hex(value);
    let printable: bool = decoded
        .iter()
        .all(|byte: &u8| { return byte.is_ascii_alphanumeric() || *byte == b'-' || *byte == b'.'; });
    if decoded.is_empty() || decoded.len() > MAX_UUID_LENGTH || !printable {
        return None;
    }
    return String::from_utf8(decoded).ok();
}

/// What:
///  The mount ID the kernel reports for a path.
/// Why:
///   It names the mount that really holds the path,
///  including when a later mount covers an ancestor's mount point,
///  which a walk over mount point names cannot see.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mountIdOf(path: string): bigint | undefined; // statx(2) stx_mnt_id
/// ```
fn mount_id_of(path: &Path) -> Option<u64> {
    use std::os::unix::ffi::OsStrExt;
    let c_path: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).ok()?;
    // SAFETY: `statx` is plain data that the call fills in, so all-zero bytes are a valid value.
    let mut buffer: libc::statx = unsafe { std::mem::zeroed() };
    // SAFETY: `c_path` is a NUL-terminated string that outlives the call, and `buffer` is a valid
    // `statx` the kernel writes at most once.
    let status: libc::c_int = unsafe {
        libc::statx(
            libc::AT_FDCWD,
            c_path.as_ptr(),
            libc::AT_NO_AUTOMOUNT | libc::AT_SYMLINK_NOFOLLOW,
            libc::STATX_MNT_ID,
            &mut buffer,
        )
    };
    if status != 0 || buffer.stx_mask & libc::STATX_MNT_ID == 0 {
        return None;
    }
    return Some(buffer.stx_mnt_id);
}

/// What:
///  The UUID text of the block device a mount source names,
///  read from the udev record libudev would read.
/// Why:
///   `findmnt` resolves the source to its canonical device node,
///  finds the sysfs block device of that name,
///  and reads `ID_FS_UUID_ENC` from `/run/udev/data/b<major>:<minor>`;
///  every step that cannot be reproduced exactly gives up.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function udevUuidOfSource(source: Uint8Array): string | undefined;
/// ```
fn udev_uuid_of_device_source(source: &[u8]) -> Option<String> {
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::{FileTypeExt, MetadataExt};
    let node: std::path::PathBuf = std::fs::canonicalize(Path::new(std::ffi::OsStr::from_bytes(source))).ok()?;
    let name: &std::ffi::OsStr = node.strip_prefix("/dev").ok()?.as_os_str();
    if name.is_empty() || name.as_bytes().contains(&b'/') {
        return None;
    }
    let metadata: std::fs::Metadata = std::fs::metadata(node.as_path()).ok()?;
    if !metadata.file_type().is_block_device() {
        return None;
    }
    let device: String = format!("{}:{}", libc::major(metadata.rdev()), libc::minor(metadata.rdev()));
    // libudev reads the device number from the sysfs entry of that name; both must agree.
    let sysfs_device: String =
        std::fs::read_to_string(Path::new(SYSFS_BLOCK_DIRECTORIES[0]).join(name).join("dev")).ok()?;
    if sysfs_device.trim_end() != device {
        return None;
    }
    let record: Vec<u8> = std::fs::read(Path::new(UDEV_DATA_DIRECTORY).join(format!("b{device}"))).ok()?;
    return udev_uuid_of_record(record.as_slice());
}

/// What:
///  The verdict for one canonical path,
///  given the text of the mount table and the mount ID the kernel reports for the path.
/// Why:
///   This is the pure decision:
///  one entry at the winning mount point,
///  the same mount the kernel reports,
///  then the source decides between a device lookup,
///  a pseudo source and unknown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mountUuidFromTable({ mountinfo, path, mountId }): MountUuid;
/// ```
pub fn mount_uuid_from_table(mountinfo: &[u8], path: &[u8], mount_id: u64) -> MountUuid {
    let well_formed: bool = path.first() == Some(&b'/')
        && !path.windows(2).any(|pair: &[u8]| { return pair == b"//"; })
        && (path.len() == 1 || !path.ends_with(b"/"));
    if !well_formed {
        return MountUuid::Unknown;
    }
    let Some(entries) = parse_mountinfo(mountinfo) else {
        return MountUuid::Unknown;
    };
    let winners: Vec<&MountEntry> = entries_at_longest_mount_point(entries.as_slice(), path);
    // Several entries on one mount point make `findmnt` print several lines.
    let [winner] = winners.as_slice() else {
        return MountUuid::Unknown;
    };
    if winner.id != mount_id {
        return MountUuid::Unknown;
    }
    if winner.source.starts_with(b"/dev/") {
        return match udev_uuid_of_device_source(winner.source.as_slice()) {
            Some(text) => MountUuid::Text(text),
            None => MountUuid::Unknown,
        };
    }
    if pseudo_source_has_no_uuid(winner.source.as_slice()) {
        return MountUuid::Absent;
    }
    return MountUuid::Unknown;
}

/// What:
///  The verdict for a canonical path on the running system.
/// Why:
///   The wrapper asks this before it starts `findmnt`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mountUuid(path: string): MountUuid;
/// ```
pub fn mount_uuid(path: &Path) -> MountUuid {
    use std::os::unix::ffi::OsStrExt;
    let Some(mount_id) = mount_id_of(path) else {
        return MountUuid::Unknown;
    };
    let Ok(mountinfo) = std::fs::read(MOUNTINFO_PATH) else {
        return MountUuid::Unknown;
    };
    return mount_uuid_from_table(mountinfo.as_slice(), path.as_os_str().as_bytes(), mount_id);
}

/// Controls for the pure decisions and the differential test against `findmnt`.
#[cfg(test)]
#[path = "filesystem_identity_mount_tests.rs"]
mod tests;
