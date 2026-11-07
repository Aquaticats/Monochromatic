//! What:
//!  The mount of a path read from `/proc/self/mountinfo`:
//! its mount point,
//! filesystem type,
//! source and both option
//!  lists,
//! so a caller can ask whether `user_subvol_rm_allowed` is active.
//! Why:
//!  The option decides whether `BTRFS_IOC_SNAP_DESTROY` works for an unprivileged user (the kernel checks the
//!  mount's super options),
//! and the doctor must tell a person which command turns it on.
//! Reading the table in the
//!  process spawns nothing.
//! `filesystem_identity_mount.rs` parses the same table but keeps only the fields it needs
//!  and declines the whole table on a shape it does not know;
//! this reader keeps the option fields and skips a line it
//!  cannot read,
//! because a missed line can only hide a mount the path is not under.
//!  The line grammar is `proc_pid_mountinfo(5)`:
//! `id parent major:minor root mount-point options [optional...] - fstype
//!  source super-options`,
//! with space,
//! tab,
//! newline and backslash escaped as `\ooo` octal in the path fields.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const facts = parseMountinfo(await readFile('/proc/self/mountinfo')).filter(m => path.startsWith(m.mountPoint)).at(-1);
//! const allowed = facts.superOptions.includes('user_subvol_rm_allowed');
//! ```

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// Where the kernel lists the mounts of this process's mount namespace.
pub const MOUNTINFO_PATH: &str = "/proc/self/mountinfo";

/// What:
///  One mount.
///  `Vec<String>` is an owned list of owned UTF-8 text (sibling `&[&str]` is a borrowed view).
/// Why:
///   The ladder and the doctor need the mount point (to name the `mount` command),
/// the type (btrfs or overlay) and
///  the options (to know whether subvolume removal is allowed).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountFacts = { mountPoint: string; filesystemType: string; source: string; mountOptions: string[]; superOptions: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MountFacts {
    /// The kernel's id of the mount (first field).
    pub mount_id: u64,
    /// The id of the mount this one is mounted in (second field);
    /// a mount hidden by a later mount at its place keeps
    ///  the id of the mount that was there before.
    pub parent_id: u64,
    /// The mount point as seen from this process's root,
    /// escapes decoded.
    pub mount_point: PathBuf,
    /// The filesystem type,
    /// such as `btrfs` or `overlay`.
    pub filesystem_type: String,
    /// The mount source,
    /// such as `/dev/dm-1`,
    /// escapes decoded.
    pub source: String,
    /// The per-mount options (sixth field),
    /// such as `rw` and `relatime`.
    pub mount_options: Vec<String>,
    /// The per-superblock options (last field),
    /// such as `compress=zstd:1` and `user_subvol_rm_allowed`.
    pub super_options: Vec<String>,
}

impl MountFacts {
    /// What:
    ///  Whether an option is active on this mount,
    /// in either option list.
    /// Why:
    ///   The kernel prints btrfs mount options such as `user_subvol_rm_allowed` among the super options,
    /// but a
    ///  per-mount option such as `ro` among the mount options;
    /// the caller should not need to know which list.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const hasOption = (m: MountFacts, name: string) => m.mountOptions.includes(name) || m.superOptions.includes(name);
    /// ```
    pub fn has_option(&self, name: &str) -> bool {
        return list_has(self.mount_options.as_slice(), name) || list_has(self.super_options.as_slice(), name);
    }
}

/// Whether one of the option texts is exactly `name`.
fn list_has(options: &[String], name: &str) -> bool {
    for option in options {
        if option == name {
            return true;
        }
    }
    return false;
}

/// Whether `byte` is a comma,
/// the separator inside an option field.
fn is_comma(byte: &u8) -> bool {
    return *byte == b',';
}

/// Whether `byte` is a space,
/// the separator between mountinfo fields.
fn is_space(byte: &u8) -> bool {
    return *byte == b' ';
}

/// Whether `byte` is a newline,
/// the separator between mountinfo lines.
fn is_newline(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// Whether `byte` is one of the octal digits `0` to `7`.
fn is_octal_digit(byte: &u8) -> bool {
    return (b'0'..=b'7').contains(byte);
}

/// What:
///  Split a comma-separated option field into its options.
/// Why:
///   Options are separated by commas;
/// an empty field has none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const options = (field: string) => field === '' ? [] : field.split(',');
/// ```
fn split_options(field: &[u8]) -> Vec<String> {
    if field.is_empty() {
        return Vec::new();
    }
    let mut options: Vec<String> = Vec::new();
    for option in field.split(is_comma) {
        options.push(String::from_utf8_lossy(option).into_owned());
    }
    return options;
}

/// What:
///  Parse one mountinfo line,
/// or nothing when it does not have the documented shape.
/// Why:
///   The fields are separated by single spaces,
/// the optional fields end at a lone `-`,
/// and three fields follow it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseLine(line: Uint8Array): MountFacts | undefined;
/// ```
fn parse_line(line: &[u8]) -> Option<MountFacts> {
    let fields: Vec<&[u8]> = line.split(is_space).collect();
    let mut separator_at: Option<usize> = None;
    for (position, field) in fields.iter().enumerate() {
        if *field == b"-" {
            separator_at = Some(position);
            break;
        }
    }
    let separator: usize = separator_at?;
    if separator < 6 || fields.len() < separator + 4 {
        return None;
    }
    return Some(MountFacts {
        mount_id: String::from_utf8_lossy(fields[0]).parse::<u64>().ok()?,
        parent_id: String::from_utf8_lossy(fields[1]).parse::<u64>().ok()?,
        mount_point: PathBuf::from(OsStr::from_bytes(unmangle_octal(fields[4]).as_slice())),
        filesystem_type: String::from_utf8_lossy(fields[separator + 1]).into_owned(),
        source: String::from_utf8_lossy(unmangle_octal(fields[separator + 2]).as_slice()).into_owned(),
        mount_options: split_options(fields[5]),
        super_options: split_options(fields[separator + 3]),
    });
}

/// What:
///  Every readable mount of the table text,
/// in file order.
/// Why:
///   File order is mount order,
/// so when two mounts share a mount point the later one is the visible one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parseMountTable = (text: Uint8Array): MountFacts[] => lines(text).map(parseLine).filter(Boolean);
/// ```
pub fn parse_mount_table(text: &[u8]) -> Vec<MountFacts> {
    return text
        .split(is_newline)
        .filter_map(parse_line)
        .collect();
}

/// What:
///  The mount whose mount point is the longest ancestor of the path (or the path itself),
/// the later one when two share a
///  mount point;
/// used only when the table's ids do not link up.
/// Why:
///   A table read from a mount namespace that hides its root,
/// or cut by a test,
/// has no chain to follow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const longest = (table: MountFacts[], path: string) => table.filter(m => isAncestor(m.mountPoint, path)).sort(byLength).at(-1);
/// ```
fn longest_mount_holding<'table>(table: &'table [MountFacts], path: &Path) -> Option<&'table MountFacts> {
    let mut best: Option<&MountFacts> = None;
    for candidate in table {
        if !path.starts_with(candidate.mount_point.as_path()) {
            continue;
        }
        let longer_or_equal: bool = match best {
            Some(current) => candidate.mount_point.as_os_str().len() >= current.mount_point.as_os_str().len(),
            None => true,
        };
        if longer_or_equal {
            best = Some(candidate);
        }
    }
    return best;
}

/// What:
///  The mount that holds `path`:
/// found by walking from the root mount down the path,
/// taking at each directory the
///  topmost mount made there inside the mount reached so far.
/// Why:
///   Walking the way path resolution does keeps a mount that a later mount covers (the covering mount's parent is the
///  same,
/// and the walk goes through the covering one) from being taken for the holder just because its mount point is
///  longer.
/// A mount whose parent id is not in the table is taken as a root of the walk;
/// when the walk reaches no mount
///  at all,
/// the longest mount point decides.
/// Comparison is by whole path components,
/// so `/var/home2` is not under
///  `/var/home`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mountOf = (table: MountFacts[], path: string) => walkFromRoot(table, path) ?? longestMount(table, path);
/// ```
pub fn mount_holding<'table>(table: &'table [MountFacts], path: &Path) -> Option<&'table MountFacts> {
    let mut known: std::collections::HashSet<u64> = std::collections::HashSet::new();
    for mount in table {
        known.insert(mount.mount_id);
    }
    let mut prefixes: Vec<&Path> = path.ancestors().collect();
    prefixes.reverse();
    let mut current: Option<&MountFacts> = None;
    for prefix in prefixes {
        loop {
            let linked: Option<&MountFacts> = topmost_linked(table, prefix, current, &known);
            // A mount whose parent is not in the table cannot be placed by ids; it is taken only when nothing placed has landed here.
            let placed_elsewhere: bool = match current {
                Some(held) => held.mount_point != prefix,
                None => false,
            };
            let unlinked: Option<&MountFacts> = if linked.is_none() && placed_elsewhere { topmost_unplaced(table, prefix, &known) } else { None };
            match linked.or(unlinked) {
                Some(found) if is_new_mount(current, found) => current = Some(found),
                _ => break,
            }
        }
    }
    if current.is_some() {
        return current;
    }
    return longest_mount_holding(table, path);
}

/// The last mount of the table made at `prefix` on top of the mount reached so far (`current`),
/// or,
/// with none reached,
/// whose
/// parent is not in the table (`known` holds every id).
fn topmost_linked<'table>(table: &'table [MountFacts], prefix: &Path, current: Option<&MountFacts>, known: &std::collections::HashSet<u64>) -> Option<&'table MountFacts> {
    for mount in table.iter().rev() {
        if mount.mount_point != prefix {
            continue;
        }
        let on_top: bool = match current {
            Some(held) => mount.parent_id == held.mount_id,
            None => !known.contains(&mount.parent_id),
        };
        if on_top {
            return Some(mount);
        }
    }
    return None;
}

/// The last mount of the table made at `prefix` whose parent id is not in the table.
fn topmost_unplaced<'table>(table: &'table [MountFacts], prefix: &Path, known: &std::collections::HashSet<u64>) -> Option<&'table MountFacts> {
    for mount in table.iter().rev() {
        if mount.mount_point == prefix && !known.contains(&mount.parent_id) {
            return Some(mount);
        }
    }
    return None;
}

/// Whether `found` is a different mount from the one reached so far,
/// so the walk moved on.
fn is_new_mount(current: Option<&MountFacts>, found: &MountFacts) -> bool {
    return match current {
        Some(held) => !std::ptr::eq(held, found),
        None => true,
    };
}

/// What:
///  The mount that holds `path` according to the kernel's table now.
///  The path is made absolute and symlink-free first,
/// because the table lists real paths.
/// Why:
///   The failure is a text a person can read:
/// the table could not be read,
/// or the path does not exist.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = mountOf(parseMountTable(await readFile(MOUNTINFO_PATH)), await realpath(path));
/// ```
pub fn read_mount_facts(path: &Path) -> Result<Option<MountFacts>, String> {
    let real: PathBuf = match std::fs::canonicalize(path) {
        Ok(resolved) => resolved,
        Err(error) => return Err(format!("cannot resolve {}: {error}", path.display())),
    };
    let text: Vec<u8> = match std::fs::read(MOUNTINFO_PATH) {
        Ok(bytes) => bytes,
        Err(error) => return Err(format!("cannot read {MOUNTINFO_PATH}: {error}")),
    };
    let table: Vec<MountFacts> = parse_mount_table(text.as_slice());
    return Ok(mount_holding(table.as_slice(), real.as_path()).cloned());
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
            && field[index + 1..index + 4].iter().all(is_octal_digit);
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

/// Mount option controls stay out of the release executable.
#[cfg(test)]
#[path = "fs_mount_options_tests.rs"]
mod tests;
