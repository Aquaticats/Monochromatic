//! The toolchain install roots of the watched user.
//! A process whose executable is under one of them is treated as a toolchain program
//! and moved outside the tunnel,
//! and this watcher runs as root while the roots sit in a directory the user can write.
//! A link is therefore never followed
//! (the owner's decision,
//! "Refuse links"):
//! a root that is a link,
//! or that lies behind a link below the home,
//! is left out of the matching and reported loudly at every start,
//! because a link to `/`,
//! to the home itself,
//! to `~/.local/share`,
//! or to another user's home would otherwise make every program of the user,
//! or of someone else,
//! leave the tunnel.
//! Plain directories are matched as before.
//! The home itself may be reached through a link,
//! such as `/home` pointing to `/var/home`,
//! because only what lies below the home is checked.

/// Escaping of untrusted names.
use crate::log_line::sanitize;
/// Names a user chose,
/// delimited so a cause ends them where the line does.
use crate::report_cause::quote_name;
/// Sorted,
/// duplicate-free names.
use std::collections::BTreeSet;
/// The opened directory handle.
use std::fs::{File, OpenOptions};
/// The descriptor number of the handle.
use std::os::fd::AsRawFd;
/// Opening a directory without following a link at its last component.
use std::os::unix::fs::OpenOptionsExt;
/// Executable paths and install roots.
use std::path::{Path, PathBuf};

/// The data directory of the toolchain below a home.
const DATA_DIRECTORY: &str = ".local/share/monochromatic-dev";
/// The install root directories inside the data directory.
const ROOT_DIRECTORIES: [&str; 2] = ["toolchain", "toolchain-global"];
/// Most names listed from the install roots in a notice.
const MAX_LISTED_NAMES: usize = 64;
/// Most directory entries read from one root,
/// so a root with a huge number of entries cannot stall the start.
const MAX_ENTRIES_READ: usize = 512;

/// A root that was left out of the matching.
#[derive(Debug, Eq, PartialEq)]
pub struct RefusedRoot {
    /// The root as it is spelled under the home.
    pub root: PathBuf,
    /// Why it was left out.
    pub why: String,
}

/// The roots the matcher compares executable paths against,
/// and the roots that were left out.
#[derive(Debug)]
pub struct InstallRoots {
    /// Root spellings accepted,
    /// compared with resolved executable paths.
    pub roots: Vec<PathBuf>,
    /// Roots left out with the reason,
    /// to be reported loudly.
    pub refused: Vec<RefusedRoot>,
}

/// What looking for links on the way to one root found.
enum LinkCheck {
    /// The root or a directory on the way to it does not exist,
    /// which is normal before the toolchain is installed.
    Absent,
    /// Every directory below the home on the way to the root,
    /// and the root,
    /// is a real directory.
    Clean,
    /// A link is on the way,
    /// or the way cannot be examined,
    /// so the root is left out.
    Refused {
        /// Why it was left out.
        why: String,
    },
}

/// Whether `target` lies strictly below `home`:
/// inside it and not the home itself,
/// and the home is not the filesystem root,
/// below which every path lies.
fn is_strictly_below(target: &Path, home: &Path) -> bool {
    return home.parent().is_some() && target != home && target.starts_with(home);
}

/// Whether a root with the same name was already refused for the same reason,
/// which happens when both spellings of the home lead to the same link.
fn already_refused(refused: &[RefusedRoot], root: &Path, why: &str) -> bool {
    for earlier in refused {
        if earlier.why == why && earlier.root.file_name() == root.file_name() {
            return true;
        }
    }
    return false;
}

/// Why a link on the way to a root is not followed,
/// naming the link below the home and where it points.
/// The words do not depend on how the home is spelled,
/// so both spellings of one link give the same reason.
fn link_reason(link: &Path, relative: &Path, is_root: bool) -> String {
    let points_to = match std::fs::read_link(link) {
        Ok(target) => format!(" (it points to {})", quote_name(&target.display().to_string())),
        Err(_error) => String::new(),
    };
    if is_root {
        return format!("it is a link; links are not followed{points_to}");
    }
    return format!("{} on the way to it is a link; links are not followed{points_to}", quote_name(&relative.display().to_string()));
}

/// Walks the directories below `base` on the way to `root`,
/// and refuses at the first link.
/// `base` itself may be reached through links,
/// such as a home that is spelled `/home/user` where `/home` points to `/var/home`.
fn check_below(base: &Path, root: &Path) -> LinkCheck {
    let Ok(relative) = root.strip_prefix(base) else {
        return LinkCheck::Refused { why: "it is not below the home".to_string() };
    };
    let mut walked = base.to_path_buf();
    let mut walked_relative = PathBuf::new();
    for component in relative.components() {
        walked.push(component);
        walked_relative.push(component);
        match std::fs::symlink_metadata(&walked) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return LinkCheck::Refused { why: link_reason(&walked, &walked_relative, walked == root) };
            }
            Ok(_other) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return LinkCheck::Absent,
            Err(error) => return LinkCheck::Refused { why: format!("it cannot be examined ({error})") },
        }
    }
    return LinkCheck::Clean;
}

/// The toolchain install roots for one home directory,
/// in every spelling a resolved executable path can take,
/// and the roots left out.
/// `home` is the passwd home of the watched user,
/// and `canonical_home` its resolved path when the directory exists:
/// a resolved executable path names the resolved home,
/// so both spellings are roots.
/// A link is never followed:
/// a root that is a link,
/// or lies behind a link below the home,
/// is left out and returned in `refused`,
/// and so is a root that cannot be examined for a reason other than not existing.
/// Roots are examined once,
/// when this runs.
pub fn install_roots(home: &Path, canonical_home: Option<&Path>) -> InstallRoots {
    let mut bases: Vec<&Path> = vec![home];
    if let Some(canonical) = canonical_home
        && canonical != home
    {
        bases.push(canonical);
    }
    let mut roots: Vec<PathBuf> = Vec::new();
    let mut refused: Vec<RefusedRoot> = Vec::new();
    for base in bases {
        for directory in ROOT_DIRECTORIES {
            let root = base.join(DATA_DIRECTORY).join(directory);
            match check_below(base, &root) {
                LinkCheck::Absent | LinkCheck::Clean => roots.push(root),
                LinkCheck::Refused { why } => {
                    if !already_refused(&refused, &root, &why) {
                        refused.push(RefusedRoot { root, why });
                    }
                }
            }
        }
    }
    return InstallRoots { roots, refused };
}

/// Lists the real directories directly inside an opened directory,
/// without dot names and at most `MAX_ENTRIES_READ` entries,
/// but only when that very directory lies strictly inside the home.
/// The place is read from the handle (`/proc/self/fd`),
/// not from the path the caller had in mind,
/// and the entries are read through the handle too,
/// so a path swapped for a link after the caller checked it cannot make this list another place.
/// Returns `None` when the handle is outside the home or cannot be read.
pub fn directory_names_from_handle(directory: &File, canonical_home: &Path) -> Option<Vec<String>> {
    let through_handle = PathBuf::from(format!("/proc/self/fd/{}", directory.as_raw_fd()));
    let real = std::fs::read_link(&through_handle).ok()?;
    if !is_strictly_below(&real, canonical_home) {
        return None;
    }
    let entries = std::fs::read_dir(&through_handle).ok()?;
    let mut names: Vec<String> = Vec::new();
    for listed in entries.take(MAX_ENTRIES_READ).flatten() {
        let Ok(kind) = listed.file_type() else {
            continue;
        };
        if !kind.is_dir() {
            continue;
        }
        let name = listed.file_name().to_string_lossy().into_owned();
        if !name.starts_with('.') {
            names.push(name);
        }
    }
    return Some(names);
}

/// Opens a directory without following a link at its last component.
fn open_directory(path: &Path) -> std::io::Result<File> {
    return OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path);
}

/// The home a root is spelled under,
/// among the given spellings.
fn base_of<'a>(root: &Path, bases: &[&'a Path]) -> Option<&'a Path> {
    for base in bases {
        if root.starts_with(base) {
            return Some(base);
        }
    }
    return None;
}

/// The names of the directories directly inside the install roots that exist,
/// sorted and without duplicates,
/// so a notice can say what is installed on this machine now.
/// A root is read only when no link is on the way to it below the home,
/// and only real directories are listed,
/// never links:
/// the roots are user-writable,
/// and this runs as root,
/// so a link must not make the notice print the names found elsewhere.
/// Each root is opened once without following a link at its last component and listed from that handle,
/// after the handle itself is checked to lie inside the home,
/// so replacing the root between the check and the read changes nothing outside the home.
/// Names are escaped before they are printed.
pub fn installed_tool_names(roots: &[PathBuf], home: &Path) -> Vec<String> {
    let Ok(canonical_home) = std::fs::canonicalize(home) else {
        return Vec::new();
    };
    let mut names: BTreeSet<String> = BTreeSet::new();
    for root in roots {
        let Some(base) = base_of(root, &[home, canonical_home.as_path()]) else {
            continue;
        };
        if !matches!(check_below(base, root), LinkCheck::Clean) {
            continue;
        }
        let Ok(directory) = open_directory(root) else {
            continue;
        };
        let Some(listed) = directory_names_from_handle(&directory, &canonical_home) else {
            continue;
        };
        for name in listed {
            names.insert(sanitize(&name));
        }
    }
    return names.into_iter().take(MAX_LISTED_NAMES).collect();
}
