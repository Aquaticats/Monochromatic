//! What:
//!  One walk of many tracked paths below a held root:
//!  each path read one plain
//!  name at a time,
//!  its parent folder reopened only when the walk moves to
//!  another folder,
//!  and the facts the capture records and the frozen proof
//!  compares.
//! Why:
//!  The capture and the proof must agree about every field they compare,
//!  and a second reader would drift.
//!  Both walks are also descriptor-relative:
//!  a folder swapped for a link while the walk runs is an error and never a
//!  detour (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
//!  "The frozen proof").
//!  Git's index is sorted by path,
//!  so the folder changes rarely and one cached
//!  descriptor serves every entry in it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! walkPaths(root, entries, (entry) => entry.path, (entry, path, found) => { ... });
//! ```

use crate::fs_beneath::{
    entry_metadata_at,
    open_directory_at,
    open_root,
    read_link_at,
};
use crate::subvolume_index::StatFields;
use crate::subvolume_index_files::fields_of;
use std::ffi::{
    OsStr,
    OsString,
};
use std::os::unix::ffi::OsStrExt as _;
use std::os::unix::fs::MetadataExt as _;
use std::path::{
    Path,
    PathBuf,
};

/// The kind of object one tracked path holds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FoundKind {
    /// A regular file.
    Regular,
    /// A symbolic link.
    Link,
    /// Nothing stands there.
    Absent,
}

impl FoundKind {
    /// The kind in words,
    /// as the capture's own record names it.
    pub(super) fn word(self) -> &'static str {
        return match self {
            FoundKind::Regular => "regular",
            FoundKind::Link => "link",
            FoundKind::Absent => "absent",
        };
    }
}

/// What:
///  One path's facts as the walk found them.
/// Why:
///  The capture records exactly these and the proof compares exactly these,
///  so a difference in any field the design names refuses the clone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FoundFacts = { kind; size; mtime; ctime; executable; linkTarget? };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct FoundFacts {
    /// What stands at the path.
    pub(super) kind: FoundKind,
    /// The fields Git's own stat comparison reads,
    /// as the capture's vouching test and the record's own facts both need them.
    /// Nothing stands at an absent path,
    /// so there are none.
    pub(super) fields: Option<StatFields>,
    /// The size in bytes.
    pub(super) size: u64,
    /// The modification time,
    /// seconds and nanoseconds.
    pub(super) mtime: (i64, u32),
    /// The change time,
    /// seconds and nanoseconds.
    pub(super) ctime: (i64, u32),
    /// Whether the owner may execute it.
    pub(super) executable: bool,
    /// A link's target bytes.
    pub(super) link_target: Option<Vec<u8>>,
}

/// What:
///  Walk one item per tracked path,
///  below a held handle of `root_path`.
/// Why:
///  The caller gives the items,
///  their paths and what to do with each,
///  so the capture and the proof share this walk and neither reads a path its own way.
///
/// # Errors
/// A path is not a plain relative path,
/// a folder on the way cannot be opened without a link,
/// or the caller's visit refused.
pub(super) fn walk_paths<'a, T: 'a>(
    root_path: &Path,
    items: impl IntoIterator<Item = &'a T>,
    path_of: impl Fn(&T) -> &[u8],
    mut visit: impl FnMut(&T, &Path, &FoundFacts, &std::fs::File, &OsStr) -> Result<(), String>,
) -> Result<(), String> {
    let root: std::fs::File = open_root(root_path).map_err(|error| {
        return format!(
            "the root {} cannot be opened without a link ({error})",
            root_path.display()
        );
    })?;
    let mut cached: Option<(PathBuf, std::fs::File)> = None;
    for item in items {
        let path: PathBuf = PathBuf::from(OsStr::from_bytes(path_of(item)));
        let names: Vec<OsString> = plain_names(path.as_path())?;
        let Some((name, parents)) = names.split_last() else {
            return Err(format!(
                "the tracked path {} names the root itself",
                path.display()
            ));
        };
        let parent_path: PathBuf = parents.iter().collect();
        let moved: bool = cached
            .as_ref()
            .map(|(cached_path, _file)| return cached_path != &parent_path)
            .unwrap_or(true);
        if moved {
            cached = Some((parent_path, open_below(&root, parents, path.as_path())?));
        }
        let Some((_cached_path, parent)) = cached.as_ref() else {
            return Err(format!("the folder of {} is not open", path.display()));
        };
        let holder: std::fs::File = parent.try_clone().map_err(|error| {
            return format!("the folder of {} cannot be held ({error})", path.display());
        })?;
        let found: FoundFacts = read_facts(&holder, name.as_os_str(), path.as_path())?;
        visit(item, path.as_path(), &found, &holder, name.as_os_str())?;
    }
    return Ok(());
}

/// The plain names of one tracked path,
/// refused when a name is `.`,
/// `..`,
/// empty or absolute.
fn plain_names(path: &Path) -> Result<Vec<OsString>, String> {
    let mut names: Vec<OsString> = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::Normal(name) => names.push(name.to_os_string()),
            _ => {
                return Err(format!(
                    "the tracked path {} is not a plain relative path",
                    path.display()
                ));
            }
        }
    }
    return Ok(names);
}

/// Open every folder of `parents` below `root`,
/// one plain name at a time.
fn open_below(
    root: &std::fs::File,
    parents: &[OsString],
    path: &Path,
) -> Result<std::fs::File, String> {
    let mut held: std::fs::File = root
        .try_clone()
        .map_err(|error| return format!("the root cannot be held ({error})"))?;
    for name in parents {
        held = open_directory_at(&held, name.as_os_str()).map_err(|error| {
            return format!("the folder of {} cannot be opened without a link ({error}); Ways forward: restore the folder the index records, or check the source out again", path.display());
        })?;
    }
    return Ok(held);
}

/// Read one path's facts below its parent's held handle.
fn read_facts(
    parent: &std::fs::File,
    name: &OsStr,
    path: &Path,
) -> Result<FoundFacts, String> {
    let metadata: std::fs::Metadata = match entry_metadata_at(parent, name) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(FoundFacts {
                kind: FoundKind::Absent,
                fields: None,
                size: 0,
                mtime: (0, 0),
                ctime: (0, 0),
                executable: false,
                link_target: None,
            });
        }
        Err(error) => return Err(format!("{} cannot be examined ({error})", path.display())),
    };
    if metadata.is_dir() {
        return Err(format!(
            "{} is a folder, which the clone route cannot prove",
            path.display()
        ));
    }
    let kind: FoundKind = if metadata.is_symlink() {
        FoundKind::Link
    } else {
        FoundKind::Regular
    };
    let link_target: Option<Vec<u8>> = match kind {
        FoundKind::Link => {
            let target: PathBuf = read_link_at(parent, name).map_err(|error| {
                return format!("the link {} cannot be read ({error})", path.display());
            })?;
            Some(target.as_os_str().as_bytes().to_vec())
        }
        _ => None,
    };
    return Ok(FoundFacts {
        kind,
        fields: Some(fields_of(&metadata)),
        size: metadata.len(),
        mtime: (metadata.mtime(), metadata.mtime_nsec() as u32),
        ctime: (metadata.ctime(), metadata.ctime_nsec() as u32),
        executable: metadata.mode() & 0o100 != 0,
        link_target,
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_capture_tree_tests.rs"]
mod tests;
