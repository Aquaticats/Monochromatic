//! What:
//! The notices about files a journal keeps beside the lockfiles:
//! what is kept,
//! how many files,
//! and where.
//! Why:
//! Split out of `journal_transaction.rs`,
//! which writes and finishes the journal of a run,
//! to keep that file under the code-line limit.
//! The items are re-exported from `journal_transaction`,
//! so their paths there are unchanged.

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
/// Imports `std::fs::{self, File, OpenOptions}` for this file.
use std::fs::{self};
// Imports `std::io`.
/// Imports `std::io` for this file.
use std::io::{self};
// Imports `std::path::{Path, PathBuf}`.
// What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
// Why:      Joined or stored paths are owned; paths passed in are borrowed.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: string
// ```
/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{Path, PathBuf};


// Imports `crate::journal::{DONE_SUFFIX, GROUPS_FILE, ...`.
/// Imports `crate::journal::{DONE_SUFFIX, GROUPS_FILE, INDEX_FILE, JOURNAL_DIR, MANIFEST_INDEX_FILE, KEPT_FILES_WARNING_COUNT, QUARANTINE_DIR, OWNER_FILE, READY_FILE, WORKSPACE_LOCK, atomic_write, create_directories_synced, make_durable, sync_directory, write_synced}` for this file.
use crate::journal::{KEPT_FILES_WARNING_COUNT, QUARANTINE_DIR};


/// What:
/// `pub fn announce_kept_notice(notice: &str)` prints the warning about the kept files,
/// and a test build traces the text.
/// Why:
/// The warning is promised on every run past the count,
/// so the emission is what a test observes,
/// not only the text the transaction
/// holds (certification,
/// second pass,
/// K1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function announceKeptNotice(notice: string): void
/// ```
pub fn announce_kept_notice(notice: &str) {
    #[cfg(test)]
    crate::bounded_run::trace(format_args!("kept notice {notice}"));
    eprintln!("{notice}");
}

/// What:
/// `fn is_kept_beside_a_lockfile(name: &str) -> bool` says whether a file name in a crate root is a kept copy or a stranded temporary
/// of this tool.
/// Why:
/// The replace protocol keeps a displaced file as `Cargo.lock.displaced*` or `Cargo.lock.refused*` beside its target when the central
/// folder cannot take it,
/// and a killed replacement leaves `.Cargo.lock.shadow-*.tmp`;
/// they are kept files too and count toward the warning (contract K1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isKeptBesideALockfile(name: string): boolean
/// ```
pub(crate) fn is_kept_beside_a_lockfile(name: &str) -> bool {
    return name.starts_with("Cargo.lock.displaced") || name.starts_with("Cargo.lock.refused") || (name.starts_with(".Cargo.lock.shadow-") && name.ends_with(".tmp"));
}

/// What:
/// `pub fn announce_kept_files_for(root: &Path, crate_roots: &[String])` prints the kept-files warning for a run that starts no
/// transaction,
/// when the inventory is past the count.
/// Why:
/// The warning is promised on every run past the count,
/// an ordinary run that finds every lockfile already satisfied included (sixth
/// pass,
/// group 7);
/// a run that begins a transaction announces through the transaction instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function announceKeptFilesFor(root: string, crateRoots: string[]): void
/// ```
pub fn announce_kept_files_for(root: &Path, crate_roots: &[String]) {
    if let Some(notice) = kept_folder_notice(root, crate_roots) {
        announce_kept_notice(&notice);
    }
}

/// What:
/// `fn kept_folder_notice(root: &Path, crate_roots: &[String]) -> Option<String>` is the warning for kept files past
/// `KEPT_FILES_WARNING_COUNT`:
/// how many (in the central folder and beside the lockfiles of the crate roots),
/// how many bytes,
/// and where to look.
/// Why:
/// Owner answer,
/// round 6,
/// "Warn past a count,
/// never refuse":
/// kept files may be someone's work,
/// so the tool neither deletes them nor stops working;
/// it tells the person on every run past the count.
/// A place that cannot be read is also reported,
/// never taken as small.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keptFolderNotice(root: string, crateRoots: string[]): string | undefined
/// ```
pub(crate) fn kept_folder_notice(root: &Path, crate_roots: &[String]) -> Option<String> {
    let directory: PathBuf = root.join(QUARANTINE_DIR);
    let mut central: usize = 0;
    let mut beside: usize = 0;
    // A wide accumulator: kept entries can have huge (sparse) lengths, and a sum that overflows would panic or wrap; the warning is
    // never refused for the size of what is kept ("Warn past a count, never refuse").
    let mut bytes: u128 = 0;
    let mut unreadable: usize = 0;
    // Whole directories that could not be listed: what is kept there is unknown, which is not "small".
    let mut unlisted: Vec<PathBuf> = Vec::new();
    match fs::read_dir(&directory) {
        Ok(listing) => {
            for entry in listing {
                match entry.and_then(metadata_of) {
                    Ok(metadata) => {
                        central += 1;
                        bytes += u128::from(metadata.len());
                    }
                    Err(_) => unreadable += 1,
                }
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => {
            return Some(format!(
                "cargo-shadow: {} could not be read ({error}), so the number of files kept there is unknown; look at it",
                directory.display()
            ));
        }
    }
    for crate_root in crate_roots {
        let place: PathBuf = root.join(crate_root);
        match fs::read_dir(&place) {
            Ok(listing) => {
                for entry in listing {
                    match entry {
                        Ok(found) => {
                            if is_kept_beside_a_lockfile(&found.file_name().to_string_lossy()) {
                                match found.metadata() {
                                    Ok(metadata) => {
                                        beside += 1;
                                        bytes += u128::from(metadata.len());
                                    }
                                    Err(_) => unreadable += 1,
                                }
                            }
                        }
                        Err(_) => unreadable += 1,
                    }
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(_) => unlisted.push(place),
        }
    }
    let count: usize = central + beside;
    let unknown: String = if unlisted.is_empty() {
        String::new()
    } else {
        format!(
            " {} could not be listed, so the number of files kept there is unknown; look at {}.",
            joined_places(&unlisted),
            if unlisted.len() == 1 { "it" } else { "them" }
        )
    };
    // An entry that could not be read may be one of any number: an iteration error can end the listing before the files that follow,
    // so an incomplete inventory is never taken as small (fourth certification pass).
    if count + unreadable <= KEPT_FILES_WARNING_COUNT && unreadable == 0 {
        if unlisted.is_empty() {
            return None;
        }
        return Some(format!("cargo-shadow:{unknown} Nothing was deleted and cargo keeps working."));
    }
    if count + unreadable <= KEPT_FILES_WARNING_COUNT {
        return Some(format!(
            "cargo-shadow: {} holds files that could not all be read ({unreadable} entries could not be read after {count} that could), so how many files are kept there is unknown; look at it.{unknown} Nothing was deleted and cargo keeps working.",
            directory.display()
        ));
    }
    let unread: String = if unreadable == 0 { String::new() } else { format!(" ({unreadable} more could not be read)") };
    let besides: String = if beside == 0 { String::new() } else { format!(" and {beside} more are kept beside the lockfiles of the crate roots (Cargo.lock.displaced*, Cargo.lock.refused*, .Cargo.lock.shadow-*.tmp)") };
    return Some(format!(
        "cargo-shadow: {} holds {central} files{besides}: {count} files, {bytes} bytes in all{unread}. They were moved aside instead of deleted because they may be someone's work. Nothing was deleted and cargo keeps working; review them when you can. Before you remove any of them, check that no program still has it open.{unknown}",
        directory.display()
    ));
}

/// What:
/// `fn metadata_of(found: fs::DirEntry) -> io::Result<fs::Metadata>` reads the metadata of a directory entry.
/// Why:
/// `Result::and_then` takes it by path instead of a closure.
pub(crate) fn metadata_of(found: fs::DirEntry) -> io::Result<fs::Metadata> {
    return found.metadata();
}

/// What:
/// `fn joined_places(places: &[PathBuf]) -> String` writes paths separated by a comma and a space.
/// Why:
/// A loop replaces the `map` and `collect` closure of the inventory message.
pub(crate) fn joined_places(places: &[PathBuf]) -> String {
    let mut texts: Vec<String> = Vec::new();
    for place in places {
        texts.push(place.display().to_string());
    }
    return texts.join(", ");
}
