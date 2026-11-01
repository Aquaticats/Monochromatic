//! What:
//!  The mount of a path read from `/proc/self/mountinfo`: its mount point, filesystem type, source and both option
//!  lists, so a caller can ask whether `user_subvol_rm_allowed` is active.
//! Why:
//!  The option decides whether `BTRFS_IOC_SNAP_DESTROY` works for an unprivileged user (the kernel checks the
//!  mount's super options), and the doctor must tell a person which command turns it on. Reading the table in the
//!  process spawns nothing. `filesystem_identity_mount.rs` parses the same table but keeps only the fields it needs
//!  and declines the whole table on a shape it does not know; this reader keeps the option fields and skips a line it
//!  cannot read, because a missed line can only hide a mount the path is not under.
//!  The line grammar is `proc_pid_mountinfo(5)`: `id parent major:minor root mount-point options [optional...] - fstype
//!  source super-options`, with space, tab, newline and backslash escaped as `\ooo` octal in the path fields.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const facts = parseMountinfo(await readFile('/proc/self/mountinfo')).filter(m => path.startsWith(m.mountPoint)).at(-1);
//! const allowed = facts.superOptions.includes('user_subvol_rm_allowed');
//! ```

use super::filesystem_identity_mount::unmangle_octal;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// Where the kernel lists the mounts of this process's mount namespace.
pub const MOUNTINFO_PATH: &str = "/proc/self/mountinfo";

/// What:
///  One mount.
///  `Vec<String>` is an owned list of owned UTF-8 text (sibling `&[&str]` is a borrowed view).
/// Why:
///   The ladder and the doctor need the mount point (to name the `mount` command), the type (btrfs or overlay) and
///  the options (to know whether subvolume removal is allowed).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountFacts = { mountPoint: string; filesystemType: string; source: string; mountOptions: string[]; superOptions: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MountFacts {
    /// The mount point as seen from this process's root, escapes decoded.
    pub mount_point: PathBuf,
    /// The filesystem type, such as `btrfs` or `overlay`.
    pub filesystem_type: String,
    /// The mount source, such as `/dev/dm-1`, escapes decoded.
    pub source: String,
    /// The per-mount options (sixth field), such as `rw` and `relatime`.
    pub mount_options: Vec<String>,
    /// The per-superblock options (last field), such as `compress=zstd:1` and `user_subvol_rm_allowed`.
    pub super_options: Vec<String>,
}

impl MountFacts {
    /// What:
    ///  Whether an option is active on this mount, in either option list.
    /// Why:
    ///   The kernel prints btrfs mount options such as `user_subvol_rm_allowed` among the super options, but a
    ///  per-mount option such as `ro` among the mount options; the caller should not need to know which list.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const hasOption = (m: MountFacts, name: string) => m.mountOptions.includes(name) || m.superOptions.includes(name);
    /// ```
    pub fn has_option(&self, name: &str) -> bool {
        return self.mount_options.iter().any(|option| { return option == name; })
            || self.super_options.iter().any(|option| { return option == name; });
    }
}

/// What:
///  Split a comma-separated option field into its options.
/// Why:
///   Options are separated by commas; an empty field has none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const options = (field: string) => field === '' ? [] : field.split(',');
/// ```
fn split_options(field: &[u8]) -> Vec<String> {
    if field.is_empty() {
        return Vec::new();
    }
    return field
        .split(|byte: &u8| { return *byte == b','; })
        .map(|option: &[u8]| { return String::from_utf8_lossy(option).into_owned(); })
        .collect();
}

/// What:
///  Parse one mountinfo line, or nothing when it does not have the documented shape.
/// Why:
///   The fields are separated by single spaces, the optional fields end at a lone `-`, and three fields follow it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseLine(line: Uint8Array): MountFacts | undefined;
/// ```
fn parse_line(line: &[u8]) -> Option<MountFacts> {
    let fields: Vec<&[u8]> = line.split(|byte: &u8| { return *byte == b' '; }).collect();
    let separator: usize = fields.iter().position(|field: &&[u8]| { return *field == b"-"; })?;
    if separator < 6 || fields.len() < separator + 4 {
        return None;
    }
    return Some(MountFacts {
        mount_point: PathBuf::from(OsStr::from_bytes(unmangle_octal(fields[4]).as_slice())),
        filesystem_type: String::from_utf8_lossy(fields[separator + 1]).into_owned(),
        source: String::from_utf8_lossy(unmangle_octal(fields[separator + 2]).as_slice()).into_owned(),
        mount_options: split_options(fields[5]),
        super_options: split_options(fields[separator + 3]),
    });
}

/// What:
///  Every readable mount of the table text, in file order.
/// Why:
///   File order is mount order, so when two mounts share a mount point the later one is the visible one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parseMountTable = (text: Uint8Array): MountFacts[] => lines(text).map(parseLine).filter(Boolean);
/// ```
pub fn parse_mount_table(text: &[u8]) -> Vec<MountFacts> {
    return text
        .split(|byte: &u8| { return *byte == b'\n'; })
        .filter_map(parse_line)
        .collect();
}

/// What:
///  The mount that holds `path`: the one with the longest mount point that is an ancestor of the path (or the path
///  itself), the later one when two share a mount point.
/// Why:
///   The comparison is by whole path components, so `/var/home2` is not under `/var/home`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mountOf = (table: MountFacts[], path: string) => table.filter(m => isAncestor(m.mountPoint, path)).sort(byLength).at(-1);
/// ```
pub fn mount_holding<'table>(table: &'table [MountFacts], path: &Path) -> Option<&'table MountFacts> {
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
///  The mount that holds `path` according to the kernel's table now.
///  The path is made absolute and symlink-free first, because the table lists real paths.
/// Why:
///   The failure is a text a person can read: the table could not be read, or the path does not exist.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = mountOf(parseMountTable(await readFile(MOUNTINFO_PATH)), await realpath(path));
/// ```
pub fn read_mount_facts(path: &Path) -> Result<Option<MountFacts>, String> {
    let real: PathBuf = std::fs::canonicalize(path)
        .map_err(|error| { return format!("cannot resolve {}: {error}", path.display()); })?;
    let text: Vec<u8> = std::fs::read(MOUNTINFO_PATH)
        .map_err(|error| { return format!("cannot read {MOUNTINFO_PATH}: {error}"); })?;
    let table: Vec<MountFacts> = parse_mount_table(text.as_slice());
    return Ok(mount_holding(table.as_slice(), real.as_path()).cloned());
}

/// Mount option controls stay out of the release executable.
#[cfg(test)]
#[path = "fs_mount_options_tests.rs"]
mod tests;
