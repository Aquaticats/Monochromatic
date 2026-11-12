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
//! the trash parent,
//! the trash root a snapshot
//!  sets its left-out directories aside in,
//! the manifest inside that root,
//! and Git's own `.git`,
//! whose index and pointer the ladder writes.
//! The trash is examined:
//! a directory or a file a person puts inside a trash root is born after the record and keeps the folder,
//! and a directory the leave-out moved in is the source's own and is not walked into,
//! because a per-file proof of what such a directory holds would read the whole ignored state of the source.
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

/// The file a trash carries inside itself,
/// written here as well as in `worktree_trash_own_manifest::MANIFEST_FILE_NAME`,
/// for the reason `TRASH_PARENT` is.
const TRASH_MANIFEST: &str = ".manifest";

/// The number of names below the trash parent in `relative`,
/// or nothing when it is not below it.
/// Why:
///   The trash's own levels are what the accounting treats differently:
/// the parent,
/// a trash root,
/// and the entries inside a root are three different answers,
/// and counting the names is how they are told apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namesBelowTheTrashParent = (relative) => relative === TRASH_PARENT ? 0 : relative.startsWith(`${TRASH_PARENT}/`) ? relative.slice(TRASH_PARENT.length + 1).split('/').length : undefined;
/// ```
fn names_below_the_trash_parent(relative: &Path) -> Option<usize> {
    let rest: &Path = relative.strip_prefix(Path::new(TRASH_PARENT)).ok()?;
    return Some(rest.components().count());
}

/// What:
///  Whether a name is one the wrapper gives a trash directory:
/// forty lowercase hexadecimal digits.
/// Why:
///   A trash root is the one directory below the trash parent that the wrapper itself makes after it records the stage,
/// so it has to be told from a directory a person keeps there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isATrashName = (name) => /^[0-9a-f]{40}$/.test(name);
/// ```
fn is_a_trash_name(name: &std::ffi::OsStr) -> bool {
    use std::os::unix::ffi::OsStrExt as _;
    let bytes: &[u8] = name.as_bytes();
    return bytes.len() == 40 && bytes.iter().all(is_a_lowercase_hexadecimal_digit);
}

/// Whether a byte is a digit or one of `a` to `f`,
/// the alphabet a trash's name is written in.
fn is_a_lowercase_hexadecimal_digit(byte: &u8) -> bool {
    return byte.is_ascii_digit() || (b'a'..=b'f').contains(byte);
}

/// What:
///  Whether the relative path is a trash root,
/// the directory one snapshot's leave-out puts its set-aside directories in.
/// Why:
///   The wrapper makes it after its record,
/// so the birth-time rule would take it for a person's entry and keep every stage whose leave-out ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isATrashRoot = (relative) => namesBelowTheTrashParent(relative) === 1 && isATrashName(basename(relative));
/// ```
fn is_a_trash_root(relative: &Path) -> bool {
    return names_below_the_trash_parent(relative) == Some(1) && relative.file_name().is_some_and(is_a_trash_name);
}

/// What:
///  Whether a name is one the leave-out gives a directory it moved into a trash:
/// `d` and a count.
/// Why:
///   Inside a trash root the wrapper's own names are the manifest and the entries it moved there,
/// and every other name is somebody's:
/// a file the wrapper never wrote has to be told from one it did,
/// and the trash records no per-file identity.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isAnEntryName = (name) => /^d[0-9]+$/.test(name);
/// ```
fn is_an_entry_name(name: &std::ffi::OsStr) -> bool {
    use std::os::unix::ffi::OsStrExt as _;
    let bytes: &[u8] = name.as_bytes();
    return bytes.len() > 1 && bytes.first() == Some(&b'd') && bytes[1..].iter().all(u8::is_ascii_digit);
}

/// What:
///  Whether the relative path is the name of an entry the leave-out moved into a trash.
/// Why:
///   The entries are what a trash holds of the source's,
/// and they are the wrapper's own by name and by the move that put them there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isAnEntryOfATrash = (relative) => namesBelowTheTrashParent(relative) === 2 && isAnEntryName(basename(relative));
/// ```
fn is_an_entry_of_a_trash(relative: &Path) -> bool {
    return names_below_the_trash_parent(relative) == Some(2) && relative.file_name().is_some_and(is_an_entry_name);
}

/// What:
///  Whether the relative path is the manifest inside a trash root,
/// the file the wrapper writes after it moved everything in.
/// Why:
///   It is the wrapper's own,
/// and it is born after the record like the trash root;
/// everything else inside a root is judged by its birth time,
/// which is what tells a directory the source had from one a person made after the kill.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isATrashManifest = (relative) => basename(relative) === TRASH_MANIFEST && isATrashRoot(dirname(relative));
/// ```
fn is_a_trash_manifest(relative: &Path) -> bool {
    let Some(parent) = relative.parent() else { return false };
    return is_a_trash_root(parent) && relative.file_name() == Some(std::ffi::OsStr::new(TRASH_MANIFEST));
}

/// What:
///  Whether the wrapper itself makes the entry at this relative path after it records the object.
/// Why:
///   The ladder writes the trash and Git's files into the stage after the record exists,
/// so those entries are the
///  wrapper's own even though they are born later;
/// everything else newborn is somebody else's.
/// `.claude` is the wrapper's own as an entry,
/// because the ladder makes it when the source
///  has none,
/// but what a person keeps inside it is still judged,
/// so only the trash below it is left unexamined.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// path === GIT_DIRECTORY || path === CLUSTER_DIRECTORY || path === TRASH_PARENT || path.startsWith(`${TRASH_PARENT}/`) || path.startsWith(`${GIT_DIRECTORY}/`)
/// ```
fn is_the_wrappers_own(relative: &Path) -> bool {
    let git: &Path = Path::new(GIT_DIRECTORY);
    let cluster: &Path = Path::new(CLUSTER_DIRECTORY);
    let trash: &Path = Path::new(TRASH_PARENT);
    return relative.starts_with(git) || relative == cluster || relative == trash || is_a_trash_root(relative) || is_a_trash_manifest(relative) || is_an_entry_of_a_trash(relative);
}

/// What:
///  Whether the entries below this relative path are not examined at all.
/// Why:
///   Git's own directory holds the source's and the ladder's entries by the thousand,
/// and nothing of the wrapper's there is worth an entry-by-entry look.
/// The trash is examined:
/// its roots and their own entries are what a person's file could stand beside or in,
/// and the two findings of the alpha.12 audit are files there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// path === GIT_DIRECTORY || path.startsWith(`${GIT_DIRECTORY}/`)
/// ```
fn is_left_unexamined(relative: &Path) -> bool {
    return relative.starts_with(GIT_DIRECTORY);
}

/// What:
///  Whether the entries below this relative path are examined and not walked into.
/// Why:
///   A directory the leave-out moved into a trash is the source's own tree,
/// moved whole;
/// a per-file proof of what it holds would read every entry of the ignored state on every add that meets a dead stage,
/// and the trash records nothing per file.
/// So the entry itself is judged by its birth time and what is inside it is left as it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// namesBelowTheTrashParent(relative) >= 2
/// ```
fn is_not_walked_into(relative: &Path) -> bool {
    return names_below_the_trash_parent(relative).is_some_and(is_deeper_than_a_trash_root);
}

/// What:
///  Whether a count of names below the trash parent is deeper than a trash root,
/// which is where a directory the leave-out moved stands.
/// Why:
///   The walk reads this answer from a number,
/// and naming it keeps the predicate about the trash's levels rather than about the number.
fn is_deeper_than_a_trash_root(below: usize) -> bool {
    return below >= 2;
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
            if is_left_unexamined(below.as_path()) {
                continue;
            }
            let metadata: std::fs::Metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Some(words_for_unexamined(below.as_path(), &error)),
            };
            // The entries the wrapper makes after its record are its own; every other one born later is not.
            if !is_the_wrappers_own(below.as_path()) {
                match metadata.created() {
                    Ok(born) if born <= written => {}
                    Ok(_) => return Some(words_for(below.as_path())),
                    Err(error) => return Some(words_for_unexamined(below.as_path(), &error)),
                }
            }
            if metadata.is_dir() && !is_not_walked_into(below.as_path()) {
                pending.push((entry.path(), below));
            }
        }
    }
    return None;
}
