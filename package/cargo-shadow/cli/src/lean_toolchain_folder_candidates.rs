//! What:
//! Which directories of the base are toolchain folders this tool may remove,
//! and the order a pass takes them in.
//! Why:
//! The listing is one level deep and each folder costs one `lstat`,
//! because a walk below a folder is what takes hours on this machine;
//! and a directory a person put beside them is not a build folder and is not this tool's to remove.
//! What is then decided about a candidate is `lean_toolchain_folders.rs`'s.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function candidates(base: string): [Candidate[], string[]]
//! ```

/// Files and directory entries.
use std::{fs, os::unix::fs::MetadataExt, path::{Path, PathBuf}};

/// The user's build base holds the toolchain folders under this name.
use crate::lean_keyed_folder::KEYED_DIRECTORY;
/// The identity of an entry.
use crate::lean_identity::Identity;
/// Directory entries and names.
use crate::lean_text::{is_directory, name_of};
/// The nanoseconds of a folder's own time,
/// which a build inside the same second moves.
use crate::lean_toolchain_folder_pieces::modified_nanos;

/// A folder the pass may remove.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Candidate {
    /// The folder.
    pub path: PathBuf,
    /// Its name under `toolchain/`.
    pub name: String,
    /// Its own modification time,
    /// in seconds,
    /// for its age and for the line it earns.
    pub modified: i64,
    /// Its own modification time in nanoseconds,
    /// because a build inside the same second moves the time and a whole second does not show it.
    pub modified_nanos: i64,
    /// Its identity when it was listed.
    pub identity: Identity,
}


/// What:
/// `fn holds_only_shards(folder: &Path) -> bool` says whether a directory holds nothing but two-hex-digit shard
/// directories,
/// or nothing at all.
/// Why:
/// Only the folders the shadow keys are removed:
/// a directory a person put under `toolchain/` is not a build folder,
/// and removing it would be removing someone's data.
/// An empty folder is a build folder whose trees all went,
/// which is what a finished removal leaves before the folder itself goes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdsOnlyShards = (folder: string): boolean => readdirSync(folder, { withFileTypes: true }).every((one) => one.isDirectory() && /^[0-9a-f]{2}$/.test(one.name));
/// ```
fn holds_only_shards(folder: &Path) -> bool {
    let Ok(entries) = fs::read_dir(folder) else { return false };
    for entry in entries.flatten() {
        let name: String = name_of(&entry.path());
        if !is_directory(&entry) || name.len() != 2 || !crate::lean_text::all_hex(&name) {
            return false;
        }
    }
    return true;
}

/// What:
/// `pub fn candidates(base: &Path) -> (Vec<Candidate>, Vec<String>)` lists the toolchain folders of the base,
/// the one modified longest ago first,
/// with one line for each folder it cannot judge.
/// Why:
/// The listing is one level deep and each folder costs one `lstat`:
/// a walk below a folder is what takes hours here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function candidates(base: string): [Candidate[], string[]]
/// ```
pub fn candidates(base: &Path) -> (Vec<Candidate>, Vec<String>) {
    let mut found: Vec<Candidate> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let Ok(entries) = fs::read_dir(base.join(KEYED_DIRECTORY)) else { return (found, notes) };
    for entry in entries.flatten() {
        if !is_directory(&entry) {
            continue;
        }
        let path: PathBuf = entry.path();
        let name: String = name_of(&path);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if holds_only_shards(&path) => found.push(Candidate {
                path,
                name,
                modified: metadata.mtime(),
                modified_nanos: modified_nanos(&metadata),
                identity: Identity { device: metadata.dev(), inode: metadata.ino() },
            }),
            Ok(_not_a_layout) => notes.push(format!("left alone {}: it holds entries that are not build shards, so it is not a layout cargo-shadow makes", path.display())),
            Err(error) => notes.push(format!("kept toolchain folder {}: its own time and identity could not be read: {error}", path.display())),
        }
    }
    found.sort_by(oldest_first);
    return (found, notes);
}

/// What:
/// `fn oldest_first(left: &Candidate, right: &Candidate) -> std::cmp::Ordering` orders folders by their own modification
/// time and then by name.
/// Why:
/// The folder modified longest ago goes first,
/// and two folders of one time keep one order between passes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const oldestFirst = (left: Candidate, right: Candidate): number => left.modified - right.modified || left.name.localeCompare(right.name);
/// ```
fn oldest_first(left: &Candidate, right: &Candidate) -> std::cmp::Ordering {
    let by_time: std::cmp::Ordering = left.modified.cmp(&right.modified);
    return match by_time {
        std::cmp::Ordering::Equal => left.name.cmp(&right.name),
        decided => decided,
    };
}
