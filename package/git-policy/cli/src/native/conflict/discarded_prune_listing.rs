//! What:
//!  Reading the folder of saved copies:
//! the directories named like the wrapper's own,
//! oldest first,
//! each with the size and identity the removal later meets,
//! and everything that could not be read,
//! said rather than skipped.
//! Why:
//!  A pass judges the folder from this listing and removes only what it saw,
//! so the listing is also where a directory that is gone,
//! replaced or unreadable is turned into a problem instead of a removal.
//! The folder is opened once without following a link,
//! and every directory is opened through that descriptor,
//! so the sizes are those of the directories a removal meets by identity
//! (`discarded_store.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { entries, problems, listedFiles } = listStamped(commonDir);
//! ```

/// The folder of saved copies inside a Git directory.
use crate::conflict_sides::SAVED_DIRECTORY;
/// The listing's shape and what a pass can say went wrong.
use crate::discarded_prune_problems::{
    Listing,
    Problem,
    ProblemKind,
};
/// A directory's name as a time, and the reading of that name.
use crate::discarded_prune_stamps::{
    Stamped,
    parse_stamp,
};
/// The opened folder, the plain values of one directory and what a walk left.
use crate::discarded_store::{
    Identity,
    Interrupted,
    OpenFailure,
    Store,
    Tally,
};
/// `OsStr` is a borrowed file name in the system's own bytes.
use std::ffi::OsStr;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};
/// What:
///  Every directory of the folder at `relative` below a Git directory,
/// oldest first,
/// with its size and what it was when
///  listed,
/// and what could not be read.
/// Why:
///   The folder is opened without following a link and each directory is opened through it,
/// so the sizes are those of the
///  directories that a later removal will meet by identity.
/// A directory that cannot be read has no size to judge by,
/// so it
///  is left out of the entries and named in the problems;
/// a folder that is absent is simply empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { entries, problems } = await listStampedBelow(commonDir, SAVED_DIRECTORY);
/// ```
pub(crate) fn list_stamped_below(
    common_dir: &Path,
    relative: &str,
) -> Listing {
    return match Store::open(common_dir, relative) {
        Ok(Some(store)) => list_in(&store, common_dir.join(relative).as_path()),
        Ok(None) => Listing::default(),
        Err(failure) => Listing {
            problems: vec![unusable(common_dir.join(relative).as_path(), &failure)],
            ..Listing::default()
        },
    };
}

/// What:
///  Every directory of the folder that is named like the wrapper's own,
/// oldest first,
/// with its size and what it was when
///  listed,
/// and what could not be read.
/// Why:
///  The pruning pass works on the folder the current name holds,
/// because it is the pass that may write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { entries, problems } = await listStamped(commonDir);
/// ```
pub fn list_stamped(common_dir: &Path) -> Listing {
    return list_stamped_below(common_dir, SAVED_DIRECTORY);
}

/// The problem that says why the folder cannot be used.
pub(crate) fn unusable(
    saved_root: &Path,
    failure: &OpenFailure,
) -> Problem {
    let detail: String = match failure {
        OpenFailure::Link => String::from(
            "a symbolic link, or something that is not a directory, stands at it or at its parent `monochromatic-git-policy-cli`, and a removal never follows one",
        ),
        OpenFailure::Refused(error) => format!("it cannot be opened: {error}"),
    };
    return Problem {
        path: saved_root.to_path_buf(),
        kind: ProblemKind::StoreUnusable,
        detail,
    };
}

/// The text of an input or output error,
/// for the places that carry an error as words.
fn io_text<T>(result: std::io::Result<T>) -> Result<T, String> {
    return match result {
        Ok(value) => Ok(value),
        Err(error) => Err(error.to_string()),
    };
}

/// Order two listed directories by the time in their names,
/// a tie broken by name.
/// `sort_by` takes this function by path.
fn compare_by_time_then_name(
    left: &Stamped,
    right: &Stamped,
) -> std::cmp::Ordering {
    return left
        .millis
        .cmp(&right.millis)
        .then(left.name.cmp(&right.name));
}

/// The listing of an opened folder;
/// `saved_root` is the path the problems and the removal records show.
pub(crate) fn list_in(
    store: &Store,
    saved_root: &Path,
) -> Listing {
    let mut listing: Listing = Listing::default();
    let names = match store.names() {
        Ok(found) => found,
        Err(error) => {
            listing.problems.push(Problem {
                path: saved_root.to_path_buf(),
                kind: ProblemKind::StoreUnusable,
                detail: format!("it cannot be read: {error}"),
            });
            return listing;
        }
    };
    for name in names {
        let text: String = name.to_string_lossy().into_owned();
        let Some(millis) = parse_stamp(text.as_str()) else {
            continue;
        };
        let path: PathBuf = saved_root.join(text.as_str());
        match survey_entry(store, name.as_os_str()) {
            Ok(None) => {}
            Ok(Some((identity, tally))) => {
                listing.files += tally.names.len();
                listing.entries.push(Stamped {
                    name: text.clone(),
                    millis,
                    bytes: tally.bytes,
                    identity,
                });
                listing
                    .listed_files
                    .insert(text, tally.names.into_iter().collect());
            }
            Err(detail) => listing.problems.push(Problem {
                path,
                kind: ProblemKind::EntryUnreadable,
                detail,
            }),
        }
    }
    listing.entries.sort_by(compare_by_time_then_name);
    return listing;
}

/// The identity and contents of the directory `name`,
/// nothing when `name` is not a directory (a link or a file),
/// or the
/// reason it cannot be read.
fn survey_entry(
    store: &Store,
    name: &OsStr,
) -> Result<Option<(Identity, Tally)>, String> {
    let Some(seen) = io_text(store.directory_identity(name))? else {
        return Ok(None);
    };
    let Some(directory) = io_text(store.open_directory(name))? else {
        return Ok(None);
    };
    let opened: Identity = io_text(directory.identity())?;
    if opened != seen {
        return Err(String::from("it was replaced while it was being read"));
    }
    return match directory.survey() {
        Ok(tally) => Ok(Some((opened, tally))),
        // Removed by another pass while it was read: nothing to count.
        Err(Interrupted {
            error,
            ..
        }) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(Interrupted {
            error,
            ..
        }) => Err(error.to_string()),
    };
}
