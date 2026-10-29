//! Filesystem reads:
//!  list a directory into an immutable,
//!  sorted snapshot.

/// What:
///  imports the ordering enum returned by comparison closures.
/// Why:
///  `sort_entries` returns `Ordering` from its comparator.
use std::cmp::Ordering;
/// What:
///  imports the standard filesystem module and its `DirEntry`/`FileType` types.
/// Why:
///  `read_directory` walks `fs::read_dir` and inspects each entry's type and metadata.
use std::fs;
/// What:
///  imports the standard I/O module for its `Result` alias.
/// Why:
///  `read_directory` surfaces the directory-open error to the caller.
use std::io;
/// What:
///  imports the borrowed filesystem-path type.
/// Why:
///  `read_directory` takes the directory to list by reference.
use std::path::Path;

/// What:
///  imports the domain entry,
///  kind,
///  and snapshot types.
/// Why:
///  reads are converted straight into these stack-agnostic values.
use crate::types::{DirectorySnapshot, EntryKind, FileEntry};

/// What:
///  read `path` into a sorted `DirectorySnapshot` tagged with `generation`.
/// Why:
///  per-entry metadata is best-effort (a failed stat yields `None` fields rather than
///      dropping the row),
///  and an unreadable single entry is skipped with a warning,
///  so one bad
///      child never fails the whole read;
///  only a failure to open the directory itself propagates.
pub fn read_directory(path: &Path, generation: u64) -> io::Result<DirectorySnapshot> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(path)? {
        match entry {
            Ok(dir_entry) => entries.push(to_file_entry(&dir_entry)),
            Err(error) => tracing::warn!(%error, "skipping unreadable directory entry"),
        }
    }
    sort_entries(&mut entries);
    return Ok(DirectorySnapshot {
        path: path.to_path_buf(),
        generation,
        entries,
    })
}

/// What:
///  convert one `read_dir` entry into a `FileEntry`.
/// Why:
///  resolves kind without following symlinks (`file_type`) and reads size/modified
///      best-effort (`metadata`),
///  lossily decoding the name so non-UTF-8 entries still show.
fn to_file_entry(entry: &fs::DirEntry) -> FileEntry {
    let metadata = entry.metadata().ok();
    return FileEntry {
        name: entry.file_name().to_string_lossy().into_owned(),
        path: entry.path(),
        kind: entry.file_type().map(kind_of).unwrap_or(EntryKind::File),
        size: metadata.as_ref().map(fs::Metadata::len),
        modified: metadata.as_ref().and_then(|meta| return meta.modified().ok()),
    }
}

/// What:
///  classify a std `FileType` into the domain `EntryKind`.
/// Why:
///  symlink is checked first because a symlink also answers `is_dir`/`is_file`
///      inconsistently across platforms;
///  an unresolved link stays its own kind.
fn kind_of(file_type: fs::FileType) -> EntryKind {
    if file_type.is_symlink() {
        return EntryKind::Symlink
    } else if file_type.is_dir() {
        return EntryKind::Directory
    } else {
        return EntryKind::File
    }
}

/// What:
///  sort entries directories-first,
///  then case-insensitively by name.
/// Why:
///  deterministic,
///  stable order so the same directory always renders identically;
///  symlinks
///      sort with files since their target is unresolved.
pub fn sort_entries(entries: &mut [FileEntry]) {
    entries.sort_by(|left, right| match (is_dir(left), is_dir(right)) {
        (true, false) => return Ordering::Less,
        (false, true) => return Ordering::Greater,
        _ => return left.name.to_lowercase().cmp(&right.name.to_lowercase()),
    });
}

/// What:
///  whether an entry sorts into the directories-first group.
/// Why:
///  true only for real directories;
///  symlinks and files share the second group.
fn is_dir(entry: &FileEntry) -> bool {
    return matches!(entry.kind, EntryKind::Directory)
}
