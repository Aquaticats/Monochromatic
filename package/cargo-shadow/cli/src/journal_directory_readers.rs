//! What:
//! The reads of a journal directory:
//! its lockfile index,
//! its ready record,
//! and the listing of the journals of a workspace.
//! Why:
//! Split out of `journal_recover.rs`,
//! which recovers the journals of killed runs,
//! to keep that file under the code-line limit.
//! The items are re-exported from `journal_recover`,
//! so their paths there are unchanged.

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
/// Imports `std::fs::{self, File}` for this file.
use std::fs;
/// Imports `std::io` for this file.
use std::io;
// Imports `std::os::unix::fs::MetadataExt`.
/// Imports `std::os::unix::fs::MetadataExt` for this file.
/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{Path, PathBuf};

/// Imports `crate::crates::{LockSnapshot, read_locks}` for this file.
use crate::crates::{LockSnapshot};
// Imports `crate::journal::{DONE_SUFFIX, INDEX_FILE, J...`.
/// Imports `crate::journal::{DONE_SUFFIX, INDEX_FILE, JOURNAL_DIR, KeepAs, LOCK_FILE, MANIFEST_INDEX_FILE, READY_FILE, REFUSED_SUFFIX, ReplaceOutcome, decode_path, file_name_text, no_hook, replace_keeping_with_hook, sync_directory}` for this file.
use crate::journal::{DONE_SUFFIX, INDEX_FILE, MANIFEST_INDEX_FILE, READY_FILE};


/// What:
/// `pub(crate) fn read_journal(directory: &Path) -> io::Result<Vec<LockSnapshot>>` reads the lockfile index and the snapshots.
/// Why:
/// The index is versioned and ends with a count,
/// so a truncated or empty index is an error and never a shorter list of roots.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readJournal(directory: string): LockSnapshot[]  // throws on a damaged journal
/// ```
pub(crate) fn read_journal(directory: &Path) -> io::Result<Vec<LockSnapshot>> {
    // Macro `format!`: builds or prints text (or a list) from a template.
    /// What:
    /// `fn corrupt(why: &str) -> io::Error` is the error for a record that is damaged:
    /// "the lockfile index is corrupt:
    /// " and the reason.
    /// Why:
    /// A function replaces the closure that every refusal of this reader called.
    fn corrupt(why: &str) -> io::Error {
        return io::Error::new(io::ErrorKind::InvalidData, format!("the lockfile index is corrupt: {why}"));
    }
    let index: String = fs::read_to_string(directory.join(INDEX_FILE))?;
    // `let mut`: a binding that may change.
    let mut lines = index.lines();
    // `Some`: the present or the absent value of an `Option`.
    if lines.next() != Some("v2") {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        return Err(corrupt("no v2 header"));
    }
    let mut snapshots: Vec<LockSnapshot> = Vec::new();
    // `None`: the present or the absent value of an `Option`.
    // `let mut`: a binding that may change.
    // Typed binding: `ended: Option<usize>`.
    let mut ended: Option<usize> = None;
    for line in lines {
        // `Some`: the present or the absent value of an `Option`.
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        if let Some(count) = line.strip_prefix("end ") {
            // `Some`: the present or the absent value of an `Option`.
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Closure `|_|`: an anonymous function, like a TS arrow function.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            let Ok(end_count) = count.trim().parse() else {
                return Err(corrupt("the end count is not a number"));
            };
            ended = Some(end_count);
            continue;
        }
        if ended.is_some() {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            return Err(corrupt("text after the end line"));
        }
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Typed binding: `source: String`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let source: String = fs::read_to_string(directory.join(format!("{}.lock", snapshots.len())))?;
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        snapshots.push(LockSnapshot { crate_dir: line.to_string(), source });
    }
    // `Some`: the present or the absent value of an `Option`.
    if ended != Some(snapshots.len()) {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        return Err(corrupt("the end count does not match the lines"));
    }
    return Ok(snapshots);
}

/// What the ready record promises.
pub(crate) struct Ready {
    /// Whether the journal carries manifest snapshots.
    pub(crate) manifests: bool,
}

/// What:
/// `pub(crate) fn read_ready(directory: &Path) -> io::Result<Option<Ready>>` reads the ready record and checks that every file it
/// lists is present with exactly the listed length;
/// `Ok(None)` means there is no ready record (the journal was never completed).
/// Why:
/// Review finding (3b 7):
/// a missing or short file of a ready journal is damage,
/// and damage keeps the journal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readReady(directory: string): Ready | undefined  // throws on a damaged journal
/// ```
pub(crate) fn read_ready(directory: &Path) -> io::Result<Option<Ready>> {
    // Macro `format!`: builds or prints text (or a list) from a template.
    /// What:
    /// `fn corrupt(why: &str) -> io::Error` is the error for a record that is damaged:
    /// "the ready record is corrupt:
    /// " and the reason.
    /// Why:
    /// A function replaces the closure that every refusal of this reader called.
    fn corrupt(why: &str) -> io::Error {
        return io::Error::new(io::ErrorKind::InvalidData, format!("the ready record is corrupt: {why}"));
    }
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `text: String`.
    let text: String = match fs::read_to_string(directory.join(READY_FILE)) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `None`: the present or the absent value of an `Option`.
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) => return Err(error),
    };
    // `let mut`: a binding that may change.
    let mut lines = text.lines();
    // `Some`: the present or the absent value of an `Option`.
    if lines.next() != Some("v1") {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        return Err(corrupt("no v1 header"));
    }
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `manifests: bool`.
    let manifests: bool = match lines.next() {
        // `Some`: the present or the absent value of an `Option`.
        Some("manifests 1") => true,
        // `Some`: the present or the absent value of an `Option`.
        Some("manifests 0") => false,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        _ => return Err(corrupt("no manifests line")),
    };
    // `let mut`: a binding that may change.
    // Typed binding: `listed: usize`.
    let mut listed: Vec<String> = Vec::new();
    let mut ended: Option<usize> = None;
    for line in lines {
        if ended.is_some() {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            return Err(corrupt("text after the end line"));
        }
        if let Some(count) = line.strip_prefix("end ") {
            let Ok(end_count) = count.trim().parse() else {
                return Err(corrupt("the end count is not a number"));
            };
            ended = Some(end_count);
            continue;
        }
        let Some((name, length)) = line.split_once('\t') else {
            return Err(corrupt("a line has no tab"));
        };
        let Ok(expected) = length.parse::<u64>() else {
            return Err(corrupt("a length is not a number"));
        };
        let found: u64 = fs::metadata(directory.join(name))?.len();
        if found != expected {
            return Err(corrupt(&format!("{name} has {found} bytes, the record says {expected}")));
        }
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        listed.push(name.to_string());
    }
    // `Some`: the present or the absent value of an `Option`.
    if ended != Some(listed.len()) {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        return Err(corrupt("no end line, or its count does not match the lines"));
    }
    // The listed set must be exactly the payload a recovery needs: the index, one snapshot per root, and the manifest index when promised.
    let roots: usize = count_index_entries(&fs::read_to_string(directory.join(INDEX_FILE))?, "v2");
    let mut required: Vec<String> = vec![INDEX_FILE.to_string()];
    for position in 0..roots {
        required.push(format!("{position}.lock"));
    }
    if manifests {
        required.push(MANIFEST_INDEX_FILE.to_string());
        let snapshots: usize = count_index_entries(&fs::read_to_string(directory.join(MANIFEST_INDEX_FILE))?, "v1");
        for position in 0..snapshots {
            required.push(format!("m{position}.toml"));
        }
    }
    // The listed files are exactly the payload the indexes imply: no omission, no duplicate, no extra.
    let mut sorted_listed: Vec<&String> = listed.iter().collect();
    sorted_listed.sort();
    let mut sorted_required: Vec<&String> = required.iter().collect();
    sorted_required.sort();
    if sorted_listed != sorted_required {
        return Err(corrupt(&format!("the listed files {listed:?} differ from the payload the indexes imply {required:?}")));
    }
    return Ok(Some(Ready { manifests }));
}

/// The journals of a workspace.
pub(crate) struct JournalListing {
    /// The journals that have not been marked done,
    /// in name order.
    pub(crate) open: Vec<PathBuf>,
    /// The journals marked done,
    /// whose removal is cleanup.
    pub(crate) done: Vec<PathBuf>,
}

/// What:
/// `pub(crate) fn list_journals(journals: &Path) -> io::Result<Option<JournalListing>>` lists the journal directories,
/// and fails when any entry cannot be listed or examined.
/// Why:
/// A survivor check and a recovery both decide from the whole set;
/// a set that could not be read completely cannot show that no earlier run left a writer
/// (round 7,
/// 6b2 1:
/// `flatten()` and `is_dir()` dropped such entries).
/// Only the opening of the journal directory finding no directory is "no journals" (`None`);
/// every failure after it,
/// `NotFound` of one entry included,
/// is an error that discards the partial listing (round 8,
/// 7b P1-2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function listJournals(journals: string): { open: string[]; done: string[] } | undefined  // throws when any entry fails
/// ```
pub(crate) fn list_journals(journals: &Path) -> io::Result<Option<JournalListing>> {
    #[cfg(test)]
    let open_fault: bool = crate::failpoints::fires(crate::failpoints::Point::ListingOpen);
    #[cfg(test)]
    let iteration_fault: bool = crate::failpoints::fires(crate::failpoints::Point::ListingIteration);
    #[cfg(test)]
    let inspection_fault: bool = crate::failpoints::fires(crate::failpoints::Point::ListingInspection);
    #[cfg(test)]
    let vanished_fault: bool = crate::failpoints::fires(crate::failpoints::Point::ListingEntryNotFound);
    // A test replaces the result of the operation, so the failure takes the same path as a failure of the kernel (round 21, sixth pass,
    // V1): the opening, the reading of an entry and its inspection each pass through the code that handles their real results.
    #[cfg(test)]
    let opened: io::Result<fs::ReadDir> = if open_fault { Err(io::Error::other("injected failure to open the journal directory")) } else { fs::read_dir(journals) };
    #[cfg(not(test))]
    let opened: io::Result<fs::ReadDir> = fs::read_dir(journals);
    let entries: fs::ReadDir = match opened {
        Ok(found) => found,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let mut listing: JournalListing = JournalListing { open: Vec::new(), done: Vec::new() };
    #[cfg_attr(not(test), allow(unused_variables))]
    for (index, found) in entries.enumerate() {
        // The second entry is the one a test makes fail, so the first is already in the partial listing.
        #[cfg(test)]
        let read: io::Result<fs::DirEntry> = if index == 1 && iteration_fault {
            crate::bounded_run::trace(format_args!("listing fault iteration at entry {index} after {} accumulated", listing.open.len() + listing.done.len()));
            Err(io::Error::other("injected failure to read an entry of the journal directory"))
        } else {
            found
        };
        #[cfg(not(test))]
        let read: io::Result<fs::DirEntry> = found;
        let entry: fs::DirEntry = read?;
        let path: PathBuf = entry.path();
        // Every entry is examined, a completed journal included: an entry that cannot be examined makes the whole listing fail.
        #[cfg(test)]
        let kind: io::Result<fs::FileType> = if index == 1 && inspection_fault {
            crate::bounded_run::trace(format_args!("listing fault inspection at entry {index} after {} accumulated", listing.open.len() + listing.done.len()));
            Err(io::Error::other("injected failure to examine an entry of the journal directory"))
        } else if index == 1 && vanished_fault {
            crate::bounded_run::trace(format_args!("listing fault entry-not-found at entry {index} after {} accumulated", listing.open.len() + listing.done.len()));
            Err(io::Error::from(io::ErrorKind::NotFound))
        } else {
            entry.file_type()
        };
        #[cfg(not(test))]
        let kind: io::Result<fs::FileType> = entry.file_type();
        let is_directory: bool = kind?.is_dir();
        if path.to_string_lossy().ends_with(DONE_SUFFIX) {
            listing.done.push(path);
        } else if is_directory {
            listing.open.push(path);
        }
    }
    listing.open.sort();
    listing.done.sort();
    return Ok(Some(listing));
}

/// What:
/// `fn count_index_entries(text: &str, header: &str) -> usize` counts the lines of an index that are neither its `end` line nor its header.
/// Why:
/// A loop replaces the `filter` closure that counted the roots and the manifest snapshots.
pub(crate) fn count_index_entries(text: &str, header: &str) -> usize {
    let mut count: usize = 0;
    for line in text.lines() {
        if !line.starts_with("end ") && line != header {
            count += 1;
        }
    }
    return count;
}
