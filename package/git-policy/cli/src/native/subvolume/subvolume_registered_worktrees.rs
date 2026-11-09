//! What:
//!  The linked worktrees of a main checkout,
//! found from `.git/worktrees/*/gitdir`,
//! and the registrations that are not one of them.
//! Why:
//!  The subvolume migration and index refresh copy and rewrite every worktree inside the checkout,
//! so the worktree of each registration is the directory its record names,
//! not the text of the record:
//! a registration written before the repository moved behind a symbolic link
//!  spells the old name and names the same directory.
//! A record whose directory is gone,
//!  or whose `.git` file Git would prune,
//! is no worktree any run may touch,
//! and every such registration is named with its reason,
//! so a run that covers fewer worktrees than are registered can say so.
//! An index is replaced as Git replaces it,
//! which `subvolume_index_files.rs` does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { inside, leftOut } = scanLinkedWorktrees(root);
//! ```

use crate::plain_file::{SMALL_FILE_LIMIT, read_plain_file};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// What:
///  One linked worktree of the checkout.
/// Why:
///   The index lives in the administrative directory;
/// the files it describes live in the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LinkedWorktree = { adminDirectory: string; worktree: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkedWorktree {
    /// `<root>/.git/worktrees/<name>`.
    pub admin_directory: PathBuf,
    /// The directory the worktree's files are in,
    /// as its real path:
    /// a registration may spell it through a symbolic link or relative to itself,
    /// and two spellings of one directory are one worktree.
    pub worktree: PathBuf,
}

/// What:
///  One registration that is not a linked worktree inside the checkout,
/// and why.
/// Why:
///   A run that covers fewer worktrees than are registered says which ones it left out,
/// so no worktree is passed over without a word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LeftOutWorktree = { adminDirectory: string; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeftOutWorktree {
    /// `<root>/.git/worktrees/<name>`.
    pub admin_directory: PathBuf,
    /// One sentence naming the worktree and why it is not covered.
    pub reason: String,
}

/// What:
///  Every registration of `root`,
/// split into the linked worktrees inside the checkout and the rest with their reasons.
/// Why:
///   See `scan_linked_worktrees`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LinkedWorktreeScan = { inside: LinkedWorktree[]; leftOut: LeftOutWorktree[] };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LinkedWorktreeScan {
    /// The worktrees whose real path is inside the checkout,
    /// sorted by path.
    pub inside: Vec<LinkedWorktree>,
    /// Every other registration,
    /// sorted by administrative directory.
    pub left_out: Vec<LeftOutWorktree>,
}

/// What:
///  The linked worktrees of `root` whose directory is inside `root` and exists.
/// Why:
///   Only worktrees inside the checkout are copied with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function linkedWorktreesInside(root: string): LinkedWorktree[];
/// ```
pub fn linked_worktrees_inside(root: &Path) -> Vec<LinkedWorktree> {
    // A listing that cannot be read is no linked worktree here; the callers that must know report it through their own steps.
    return match scan_linked_worktrees(root) {
        Ok(scan) => scan.inside,
        Err(_unreadable) => Vec::new(),
    };
}

/// What:
///  Every registration under `root/.git/worktrees`,
/// split into the linked worktrees inside the checkout and the rest.
/// Why:
///   Whether a registered worktree is inside the checkout is a question about the file the registration names,
/// not about its text:
/// a registration written before the repository moved behind a symbolic link spells the old name
///  and names a worktree inside the checkout all the same.
/// Each registration that is not covered carries the reason,
/// so a caller can say why it looked at fewer worktrees than are registered.
/// A `gitdir` file Git skips (unreadable or empty)
///  belongs to a worktree Git would prune,
/// and a record whose directory is gone is one Git would prune too;
/// both are named in `left_out`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scanLinkedWorktrees(root: string): LinkedWorktreeScan; // throws when the registrations cannot be listed
/// ```
pub fn scan_linked_worktrees(root: &Path) -> Result<LinkedWorktreeScan, String> {
    let mut scan: LinkedWorktreeScan = LinkedWorktreeScan::default();
    let admin: PathBuf = root.join(".git").join("worktrees");
    let names = match std::fs::read_dir(&admin) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(scan),
        // A run that cannot read the registrations would look at none of them and say nothing, so it is an error.
        Err(error) => return Err(format!("the registered worktrees of {} cannot be listed ({error}), so how many there are cannot be known", admin.display())),
    };
    // A root that cannot be resolved is compared as given, as the registration's worktree is.
    let real_root: PathBuf = match std::fs::canonicalize(root) {
        Ok(resolved) => resolved,
        Err(_unresolved) => root.to_path_buf(),
    };
    for name in names.flatten() {
        let admin_directory: PathBuf = name.path();
        let recorded: Vec<u8> = match read_plain_file(&admin_directory.join("gitdir"), SMALL_FILE_LIMIT) {
            Ok(bytes) => bytes,
            Err(error) => {
                scan.left_out.push(left_out(admin_directory.as_path(), format!("the registration {} has no readable gitdir file ({error}), so Git would prune it", admin_directory.display())));
                continue;
            }
        };
        // The recorded path is raw bytes: a directory named with bytes that are not UTF-8 is a valid path on Linux.
        let line: &[u8] = recorded.trim_ascii();
        if line.is_empty() {
            scan.left_out.push(left_out(admin_directory.as_path(), format!("the registration {} has an empty gitdir file, so Git would prune it", admin_directory.display())));
            continue;
        }
        let marker: PathBuf = recorded_marker(admin_directory.as_path(), line);
        let Some(named) = marker.parent() else {
            scan.left_out.push(left_out(admin_directory.as_path(), format!("the registration {} names {}, which has no directory to hold a worktree", admin_directory.join("gitdir").display(), marker.display())));
            continue;
        };
        let worktree: PathBuf = match std::fs::canonicalize(named) {
            Ok(resolved) => resolved,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                scan.left_out.push(left_out(admin_directory.as_path(), format!("the registered worktree {} is gone, so Git would prune the registration in {}", named.display(), admin_directory.display())));
                continue;
            }
            Err(error) => {
                scan.left_out.push(left_out(admin_directory.as_path(), format!("the registered worktree {} cannot be resolved ({error})", named.display())));
                continue;
            }
        };
        if !worktree.is_dir() {
            scan.left_out.push(left_out(admin_directory.as_path(), format!("the registered worktree {} is not a directory", worktree.display())));
            continue;
        }
        if worktree == real_root {
            scan.left_out.push(left_out(admin_directory.as_path(), format!("the registration {} names the main checkout {} itself", admin_directory.display(), real_root.display())));
            continue;
        }
        if !worktree.starts_with(real_root.as_path()) {
            scan.left_out.push(left_out(admin_directory.as_path(), format!("the registered worktree {} is outside the main checkout {}", worktree.display(), real_root.display())));
            continue;
        }
        scan.inside.push(LinkedWorktree { admin_directory, worktree });
    }
    scan.inside.sort_by(by_worktree_path);
    scan.left_out.sort_by(by_admin_directory);
    return Ok(scan);
}

/// One registration left out of a scan.
fn left_out(admin_directory: &Path, reason: String) -> LeftOutWorktree {
    return LeftOutWorktree { admin_directory: admin_directory.to_path_buf(), reason };
}

/// What:
///  The `.git` file a registration's record names,
/// absolute whether the record is absolute or written relative to the registration.
/// Why:
///   Git writes the record relative to the registration's directory under `--relative-paths` and `worktree.useRelativePaths`,
/// and the worktree is its parent either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const marker = isAbsolute(recorded) ? recorded : join(adminDirectory, recorded);
/// ```
fn recorded_marker(admin_directory: &Path, recorded: &[u8]) -> PathBuf {
    let named: &Path = Path::new(OsStr::from_bytes(recorded));
    return if named.is_absolute() { named.to_path_buf() } else { admin_directory.join(named) };
}

/// Order two linked worktrees by their path.
fn by_worktree_path(left: &LinkedWorktree, right: &LinkedWorktree) -> std::cmp::Ordering {
    return left.worktree.cmp(&right.worktree);
}

/// Order two left-out registrations by their administrative directory.
fn by_admin_directory(left: &LeftOutWorktree, right: &LeftOutWorktree) -> std::cmp::Ordering {
    return left.admin_directory.cmp(&right.admin_directory);
}
