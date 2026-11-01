//! Core domain types for the file-manager state,
//!  kept free of GTK so they unit-test in isolation.

/// What:
///  imports the owned filesystem-path type.
/// Why:
///  every entry and pane location carries an absolute `PathBuf`.
use std::path::PathBuf;
/// What:
///  imports the wall-clock timestamp type.
/// Why:
///  a directory entry carries its last-modified time for display and sorting.
use std::time::SystemTime;

/// What:
///  stable identity for one pane instance,
///  a newtype over a monotonic counter.
/// Why:
///  a pane survives deliberate duplicates,
///  so identity is never merely its location;
///  the id
///      is the dedup-independent handle used by the column layout and the focus state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PaneId(
    /// Raw monotonic counter value backing the identity.
    pub u64,
);

/// What:
///  classification of a filesystem entry as reported by the read,
///  without following links.
/// Why:
///  click behavior and sort order branch on it (a directory spawns a listing,
///  a file a
///      preview);
///  the target kind of a symlink is intentionally left unresolved at read time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryKind {
    /// Directory:
    ///  single-click spawns a listing pane rooted here.
    Directory,
    /// Regular file:
    ///  single-click spawns a preview pane;
    ///  Enter/double-click opens it.
    File,
    /// Symbolic link:
    ///  its target kind is not resolved when the directory is read.
    Symlink,
}

/// What:
///  what a pane shows;
///  also the dedup lookup key.
/// Why:
///  two panes with equal locations are the same pane unless a duplicate was explicitly
///      forced,
///  so equality/hash over the location drives dedup-and-focus on revisit.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PaneLocation {
    /// A directory listing rooted at this path.
    Directory(
        /// Absolute path of the listed directory.
        PathBuf,
    ),
    /// A preview of the single file at this path.
    Preview(
        /// Absolute path of the previewed file.
        PathBuf,
    ),
}

/// What:
///  one row of a directory listing:
///  display name,
///  absolute path,
///  kind,
///  and best-effort
///       metadata (absent when the per-entry stat failed).
/// Why:
///  the immutable unit the listing pane renders and the click handler spawns from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileEntry {
    /// Final path segment shown to the user.
    pub name: String,
    /// Absolute path used to open or spawn from this entry.
    pub path: PathBuf,
    /// Directory/file/symlink classification driving click behavior and sort.
    pub kind: EntryKind,
    /// Size in bytes when known;
    ///  `None` when the entry could not be stat-ed.
    pub size: Option<u64>,
    /// Last-modified time when known;
    ///  `None` when the entry could not be stat-ed.
    pub modified: Option<SystemTime>,
}

/// What:
///  an immutable listing of one directory at a single read generation.
/// Why:
///  evictable and re-readable;
///  the generation lets a newer read supersede a stale snapshot
///      without the UI confusing the two.
#[derive(Clone, Debug)]
pub struct DirectorySnapshot {
    /// Directory this snapshot lists.
    pub path: PathBuf,
    /// Monotonic read generation;
    ///  a higher value is a newer read of the same directory.
    pub generation: u64,
    /// Entries already sorted (directories first,
    ///  then case-insensitive by name).
    pub entries: Vec<FileEntry>,
}
