//! What:
//!  Reading the source and making the destination's directories:
//! the walk that names every entry,
//! the directories it makes before any file is placed,
//! the modes the source's directories keep,
//! and the three job shapes the blocking pool takes them in.
//! Why:
//!  Directories must exist before a worker places a file in them,
//! and modes go last so a read-only directory does not stop its own contents from being placed;
//! both facts are one order of steps,
//! so they live beside each other,
//! and a job of the pool owns what it reads rather than a closure.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function walk(source: string): Entry[];
//! type TheWalk = { entries: Entry[]; leftOut: string[]; directories: number[]; rootMode: Permissions };
//! function walkAndMakeDirectories(source: string, destination: string): TheWalk;
//! function setTheModes(source: string, destination: string, entries: Entry[], directories: number[], rootMode: Permissions): void;
//! ```

/// Import the two words a failed step is said in.
use crate::package_repair_tree_clone_entries_and_places::say_where;
/// Import what a walk of the source finds at one path,
/// the entry it records,
/// and the order it records them in.
use crate::package_repair_tree_clone_entries_and_places::{
    Entry,
    Kind,
    compare_by_relative_path,
};
/// Import the metadata a directory's mode is read from.
use std::fs::Metadata;
/// Import the permission bits a directory keeps.
use std::fs::Permissions;
/// Import the borrowed and owned filesystem paths a walk names.
use std::path::{
    Path,
    PathBuf,
};
/// Import the shared entry list one job reads from.
use std::sync::Arc;
/// What:
///  Every entry below the source,
/// parents before children,
/// without following links.
/// Why:
///   Directories must exist before a worker places a file in them,
/// and a link to a directory must be copied as a link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function walk(source: string): Entry[];
/// ```
pub(super) fn walk(source: &Path) -> Result<(Vec<Entry>, Vec<PathBuf>), String> {
    let mut found: Vec<Entry> = Vec::new();
    let mut left_out: Vec<PathBuf> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![PathBuf::new()];
    while let Some(relative) = pending.pop() {
        let directory: PathBuf = source.join(&relative);
        let listing = match std::fs::read_dir(&directory) {
            Ok(listed_directory) => listed_directory,
            // A directory a test or a tool left without read permission has nothing the clone can take; it is made empty and named.
            Err(error)
                if error.kind() == std::io::ErrorKind::PermissionDenied
                    && !relative.as_os_str().is_empty() =>
            {
                left_out.push(relative);
                continue;
            }
            Err(error) => return Err(format!("{} cannot be read ({error})", directory.display())),
        };
        for listed in listing {
            let item = say_where(listed, directory.as_path(), "cannot be listed")?;
            let path: PathBuf = relative.join(item.file_name());
            let file_type = say_where(item.file_type(), item.path().as_path(), "has no type")?;
            if file_type.is_dir() {
                pending.push(path.clone());
                found.push(Entry {
                    relative: path,
                    kind: Kind::Directory,
                });
            } else if file_type.is_symlink() {
                found.push(Entry {
                    relative: path,
                    kind: Kind::Link,
                });
            } else if file_type.is_file() {
                found.push(Entry {
                    relative: path,
                    kind: Kind::File,
                });
            }
        }
    }
    // Parents sort before their children, so directories can be made in list order.
    found.sort_by(compare_by_relative_path);
    left_out.sort();
    return Ok((found, left_out));
}

/// What:
///  The walk and the making of the directories as the state of one job.
/// Why:
///   Same as `place_a_slice_of`:
/// the walk is storage work,
/// and its two paths travel as values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const walkAndMake = ([source, destination]) => walkAndMakeDirectories(source, destination);
/// ```
pub(super) fn walk_and_make_of(
    (source, destination): (PathBuf, PathBuf)
) -> Result<TheWalk, String> {
    return walk_and_make_directories(source.as_path(), destination.as_path());
}

/// What:
///  The modes of a finished clone as the state of one job.
/// Why:
///   Same as `place_a_slice_of`:
/// the modes are storage work,
/// and everything the job reads travels as values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const setTheModesOf = ([source, destination, entries, directories, rootMode]) => setTheModes(source, destination, entries, directories, rootMode);
/// ```
pub(super) fn set_the_modes_of(
    (source, destination, entries, directories, root_mode): (
        PathBuf,
        PathBuf,
        Arc<[Entry]>,
        Vec<usize>,
        Permissions,
    )
) -> Result<(), String> {
    return set_the_modes(
        source.as_path(),
        destination.as_path(),
        entries.as_ref(),
        directories.as_slice(),
        root_mode,
    );
}

/// What:
///  Everything a clone does before and after its slices,
/// as the pool does it.
/// Why:
///   The walk,
/// the directory creation and the modes are storage waits,
/// so they run as jobs of the blocking pool rather than on the task
///  that awaits them (`doc/planning/git-policy-cli-full-tokio.md`,
/// "Blocking files,
/// fsync and the runtime's own shutdown").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheWalk = { entries: Entry[]; leftOut: string[]; directories: number[]; rootMode: Permissions };
/// ```
pub(super) struct TheWalk {
    /// Every entry below the source,
    /// parents before children.
    pub(super) entries: Vec<Entry>,
    /// The paths that could not be read and so are not in the destination.
    pub(super) left_out: Vec<PathBuf>,
    /// The positions of the entries that are directories,
    /// which are made now and whose modes are set at the end.
    pub(super) directories: Vec<usize>,
    /// The mode the source itself has,
    /// which the destination takes last.
    pub(super) root_mode: Permissions,
}

/// What:
///  Walk the source and make every directory of the destination,
/// so the slices have somewhere to place their files.
/// Why:
///   Directories must exist before a slice places a file in them,
/// and the walk names the entry it could not read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function walkAndMakeDirectories(source: string, destination: string): TheWalk;
/// ```
pub(super) fn walk_and_make_directories(
    source: &Path,
    destination: &Path,
) -> Result<TheWalk, String> {
    let (entries, left_out): (Vec<Entry>, Vec<PathBuf>) = walk(source)?;
    let root: Metadata = say_where(std::fs::metadata(source), source, "cannot be read")?;
    if let Some(parent) = destination.parent() {
        say_where(std::fs::create_dir_all(parent), parent, "cannot be made")?;
    }
    // Exclusive creation: an existing destination is refused and left as it was.
    say_where(
        std::fs::create_dir(destination),
        destination,
        "cannot be made",
    )?;
    let mut directories: Vec<usize> = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        if entry.kind == Kind::Directory {
            let made: PathBuf = destination.join(&entry.relative);
            say_where(std::fs::create_dir(&made), made.as_path(), "cannot be made")?;
            directories.push(index);
        }
    }
    return Ok(TheWalk {
        entries,
        left_out,
        directories,
        root_mode: root.permissions(),
    });
}

/// What:
///  Give the destination and its directories the modes of the source,
/// the deepest directory first.
/// Why:
///   Modes go last,
/// so a read-only directory does not stop its own contents from being placed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function setTheModes(source: string, destination: string, entries: Entry[], directories: number[], rootMode: Permissions): void;
/// ```
pub(super) fn set_the_modes(
    source: &Path,
    destination: &Path,
    entries: &[Entry],
    directories: &[usize],
    root_mode: Permissions,
) -> Result<(), String> {
    for at in directories.iter().rev() {
        let Some(entry) = entries.get(*at) else {
            continue;
        };
        let made: PathBuf = destination.join(&entry.relative);
        let source_directory: Metadata = say_where(
            std::fs::metadata(source.join(&entry.relative)),
            entry.relative.as_path(),
            "cannot be read",
        )?;
        say_where(
            std::fs::set_permissions(&made, source_directory.permissions()),
            made.as_path(),
            "cannot take its mode",
        )?;
    }
    say_where(
        std::fs::set_permissions(destination, root_mode),
        destination,
        "cannot take its mode",
    )?;
    return Ok(());
}
