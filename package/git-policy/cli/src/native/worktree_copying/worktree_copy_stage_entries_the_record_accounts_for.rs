//! What:
//!  The first entry of a stage container that the record of the object its creation made cannot account for,
//! and why,
//! as one sentence a person reads.
//! Why:
//!  A record of that object proves which directory the wrapper made;
//! it says nothing about what stands inside it,
//! and a person who
//!  puts a file there after the wrapper died must not lose it to the reaping.
//! The record names the instant after which nothing of the wrapper's is written into the container,
//! so every entry
//!  born after it is unaccounted for,
//! except the entries the wrapper itself makes later and names here:
//! the trash a snapshot
//!  sets its left-out directories aside in,
//! and Git's own `.git`,
//! whose index and pointer the ladder writes.
//! A folder holding an unaccounted entry is left whole and named,
//! which is the conservative answer for a
//!  folder that holds a copy of a checkout's state.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const unaccounted = firstEntryTheRecordDoesNotAccountFor(container, recordedInstant); // undefined when none
//! ```

use crate::worktree_copy_repository_path::shown_repository_path;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// The relative path of the directory a snapshot's leave-out moves its set-aside directories into.
/// Why it is written here as well as in `worktree_trash::TRASH_PARENT`:
///  that module is built only on Linux,
/// and this one is built wherever the reaping is;
/// the two names are one
///  path,
/// and a snapshot stage exists only on Linux,
/// where both must name it the same way.
const TRASH_PARENT: &str = ".claude/cli-git-trash";

/// The directory the snapshot of a source carries its git directory in.
const GIT_DIRECTORY: &str = ".git";

/// The directory the cluster tooling of this repository keeps its files in,
/// which holds the trash.
const CLUSTER_DIRECTORY: &str = ".claude";

/// What:
///  Whether the wrapper itself makes the entry at this relative path after it records the object.
/// Why:
///   The ladder writes the trash and Git's files into the stage after the record exists,
/// so those entries are the
///  wrapper's own even though they are born later;
/// everything else newborn is somebody else's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// path === GIT_DIRECTORY || path === CLUSTER_DIRECTORY || path === TRASH_PARENT || path.startsWith(`${TRASH_PARENT}/`) || path.startsWith(`${GIT_DIRECTORY}/`)
/// ```
fn is_the_wrappers_own(relative: &Path) -> bool {
    let git: &Path = Path::new(GIT_DIRECTORY);
    let cluster: &Path = Path::new(CLUSTER_DIRECTORY);
    let trash: &Path = Path::new(TRASH_PARENT);
    return relative.starts_with(git) || relative == cluster || relative == trash || relative.starts_with(trash);
}

/// What:
///  Whether a listed entry's name is anything at all,
/// which every name is but the empty one.
/// Why:
///   A directory listed through `/proc` answers with its own entries,
/// and a name that is empty would name the directory itself,
/// which is never an entry below it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// name.length > 0
/// ```
fn is_a_name(name: &std::ffi::OsStr) -> bool {
    use std::os::unix::ffi::OsStrExt as _;
    return !name.as_bytes().is_empty();
}

/// What:
///  The words for one entry the record does not account for.
/// Why:
///   The person is sent to one path they can look at,
/// and a folder that cannot be listed is named the same way,
/// so one
///  sentence covers both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `it holds ${shown}, which the record of the object it made does not account for`
/// ```
fn words_for(relative: &Path) -> String {
    use std::os::unix::ffi::OsStrExt as _;
    return format!(
        "it holds {}, which the record of the object it made does not account for",
        shown_repository_path(relative.as_os_str().as_bytes())
    );
}

/// What:
///  The words for an entry or a directory that cannot be examined,
/// with the reason.
/// Why:
///   A look that failed is not an answer of absence,
/// so the folder is kept and the person reads what could not be
///  read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `nothing shows what ${shown} is (${reason}), so nothing below it was examined`
/// ```
fn words_for_unexamined(relative: &Path, error: &std::io::Error) -> String {
    use std::os::unix::ffi::OsStrExt as _;
    return format!(
        "nothing shows what {} is ({error}), so nothing below it was examined and none of it was removed",
        shown_repository_path(relative.as_os_str().as_bytes())
    );
}

/// What:
///  The entries born after the recorded instant that the record does not name,
/// as the first one found,
/// and the reason it is kept,
/// or nothing when every entry is accounted for.
/// Why:
///   The reaping removes a stage only when this answers nothing;
/// anything else keeps the folder and is said to the person,
///  because a person's entry there exists nowhere else and the wrapper's own is a copy.
/// The walk reads and never writes,
/// opens no file,
///  and never follows a link,
/// so a stage whose entries are many costs lookups and no waiting;
/// a directory that cannot be listed ends the
///  walk with its reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unaccounted = firstEntryTheRecordDoesNotAccountFor(container, written);
/// ```
pub fn unaccounted_entry(container: &Path, written: SystemTime) -> Option<String> {
    let mut pending: Vec<(PathBuf, PathBuf)> = vec![(container.to_path_buf(), PathBuf::new())];
    while let Some((directory, relative)) = pending.pop() {
        let listing = match std::fs::read_dir(directory.as_path()) {
            Ok(listing) => listing,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Some(words_for_unexamined(relative.as_path(), &error)),
        };
        for found in listing {
            let entry: std::fs::DirEntry = match found {
                Ok(entry) => entry,
                Err(error) => return Some(words_for_unexamined(relative.as_path(), &error)),
            };
            let name: std::ffi::OsString = entry.file_name();
            if !is_a_name(name.as_os_str()) {
                continue;
            }
            let below: PathBuf = relative.join(name.as_os_str());
            if is_the_wrappers_own(below.as_path()) {
                continue;
            }
            let metadata: std::fs::Metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Some(words_for_unexamined(below.as_path(), &error)),
            };
            match metadata.created() {
                Ok(born) if born <= written => {}
                Ok(_) => return Some(words_for(below.as_path())),
                Err(error) => return Some(words_for_unexamined(below.as_path(), &error)),
            }
            if metadata.is_dir() {
                pending.push((entry.path(), below));
            }
        }
    }
    return None;
}
