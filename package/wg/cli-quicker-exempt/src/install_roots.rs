//! The toolchain install roots of the watched user.
//! A process whose executable is under one of them is treated as a toolchain program
//! and moved outside the tunnel,
//! and this watcher runs as root while the roots sit in a directory the user can write.
//! A root that is a link therefore counts as the place it points to
//! only when that place lies strictly inside the user's resolved home:
//! a link to `/`,
//! to the home itself,
//! or to another user's home would otherwise make every program of the user,
//! or of someone else,
//! leave the tunnel.

/// Escaping of untrusted names.
use crate::log_line::sanitize;
/// Sorted,
/// duplicate-free names.
use std::collections::BTreeSet;
/// Executable paths and install roots.
use std::path::{Path, PathBuf};

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

/// What resolving one root led to.
enum Resolution {
    /// The root does not exist,
    /// which is normal before the toolchain is installed.
    Absent,
    /// The root is its own resolved path.
    Plain,
    /// The root is a link to a place strictly inside the home.
    Accepted {
        /// The resolved place the link points to.
        target: PathBuf,
    },
    /// The root was left out.
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

/// Resolves one root and decides whether the place it points to may count.
fn resolve_root(root: &Path, canonical_home: Option<&Path>) -> Resolution {
    let target = match std::fs::canonicalize(root) {
        Ok(target) => target,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Resolution::Absent,
        Err(error) => return Resolution::Refused { why: format!("it cannot be resolved ({error})") },
    };
    if target == root {
        return Resolution::Plain;
    }
    let Some(home) = canonical_home else {
        return Resolution::Refused {
            why: format!(
                "it points to {}, and the home directory cannot be resolved, so the link cannot be checked",
                target.display()
            ),
        };
    };
    if is_strictly_below(&target, home) {
        return Resolution::Accepted { target };
    }
    return Resolution::Refused {
        why: format!(
            "it points to {}, which is not strictly inside the home {}",
            target.display(),
            home.display()
        ),
    };
}

/// The toolchain install roots for one home directory,
/// in every spelling a resolved executable path can take,
/// and the roots left out.
/// `home` is the passwd home of the watched user,
/// and `canonical_home` its resolved path when the directory exists.
/// A root that is a link is also known by the place it points to,
/// because a resolved executable path names that place,
/// but only when that place lies strictly inside `canonical_home`.
/// Any other link,
/// and a root that cannot be resolved for a reason other than not existing,
/// is left out and returned in `refused`.
/// Roots are resolved once,
/// when this runs.
pub fn install_roots(home: &Path, canonical_home: Option<&Path>) -> InstallRoots {
    let mut spelled: Vec<PathBuf> = Vec::new();
    let mut homes: Vec<&Path> = vec![home];
    if let Some(canonical) = canonical_home {
        homes.push(canonical);
    }
    for candidate in homes {
        for directory in ["toolchain", "toolchain-global"] {
            let root = candidate.join(".local/share/monochromatic-dev").join(directory);
            if !spelled.contains(&root) {
                spelled.push(root);
            }
        }
    }
    let mut roots = spelled.clone();
    let mut refused: Vec<RefusedRoot> = Vec::new();
    for root in &spelled {
        match resolve_root(root, canonical_home) {
            Resolution::Absent | Resolution::Plain => {}
            Resolution::Accepted { target } => {
                if !roots.contains(&target) {
                    roots.push(target);
                }
            }
            Resolution::Refused { why } => {
                if !already_refused(&refused, root, &why) {
                    refused.push(RefusedRoot { root: root.clone(), why });
                }
            }
        }
    }
    return InstallRoots { roots, refused };
}

/// The names of the directories directly inside the install roots that exist,
/// sorted and without duplicates,
/// so a notice can say what is installed on this machine now.
/// A root is read only when it resolves to a place inside the user's home,
/// and only real directories are listed,
/// never links:
/// the roots are user-writable,
/// and this runs as root,
/// so a link must not make the notice print the names found elsewhere.
/// Names are escaped before they are printed.
pub fn installed_tool_names(roots: &[PathBuf], home: &Path) -> Vec<String> {
    let Ok(canonical_home) = std::fs::canonicalize(home) else {
        return Vec::new();
    };
    let mut names: BTreeSet<String> = BTreeSet::new();
    for root in roots {
        let Ok(canonical) = std::fs::canonicalize(root) else {
            continue;
        };
        if !is_strictly_below(&canonical, &canonical_home) {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&canonical) else {
            continue;
        };
        for listed in entries.take(MAX_ENTRIES_READ).flatten() {
            let Ok(kind) = listed.file_type() else {
                continue;
            };
            if !kind.is_dir() {
                continue;
            }
            let name = listed.file_name().to_string_lossy().into_owned();
            if !name.starts_with('.') {
                names.insert(sanitize(&name));
            }
        }
    }
    return names.into_iter().take(MAX_LISTED_NAMES).collect();
}
