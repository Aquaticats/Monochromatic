//! The questions the refresh planner asks the disk, behind a trait so a test can answer them without a real library.
//!
//! What: `Disk` answers what one path is, what a folder holds directly, and which tracks a whole subtree holds in
//! queue order; `RealDisk` answers from the file system with the same rules as the folder walk.
//! Why: the planner must be tested with a million tracks, and a million files on disk are too slow to create in the
//! normal suite; the trait lets the large test answer from memory while the small tests use real files.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! interface Disk { kind(path: string): EntryKind; entries(folder: string): [string, EntryKind][]; walk(folder: string): string[]; }
//! ```

/// What: `use std::path::{Path, PathBuf};` imports two path types: `Path`, a borrowed read-only view of a path, and
/// `PathBuf`, an owned and growable path. Siblings: `&str` and `String` are the text-only counterparts, which cannot
/// hold file names that are not valid UTF-8.
/// Why: the trait methods look at paths they are given (`&Path`) and hand back paths they found (`PathBuf`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Paths are plain strings in TypeScript; there is no Path or PathBuf to import.
/// ```
use std::path::{Path, PathBuf};

/// What: `use crate::playback::is_audio_file;` imports the audio-extension test from `src/playback.rs`. `crate::` means
/// "from the root of this crate".
/// Why: a file is a track only if its extension is in the audio allowlist, which the folder walk also uses, so both agree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { isAudioFile } from "./playback";
/// ```
use crate::playback::is_audio_file;

/// What: `pub(crate) enum EntryKind { Missing, Folder, Track, Other }` is a fixed set of four named cases, a tagged union
/// whose cases carry no data. `pub(crate)` makes it visible inside this crate only (siblings: plain `pub` reaches other
/// crates; no keyword is private to the file). `#[derive(Clone, Copy, Debug, PartialEq, Eq)]` asks the compiler to write
/// five standard abilities: `Clone` and `Copy` (a value is duplicated by copying bits, so passing it never uses it up),
/// `Debug` (printing with `{:?}`), and `PartialEq` and `Eq` (comparing with `==`).
/// Why: the queue lists audio files; a folder leads to more of them, and anything else (a missing path, a text file, a
/// sidecar) is not listed. Deriving `Copy` makes the value behave like a number: the callers compare and pass it freely.
/// Gotcha: without `PartialEq`, `kind == EntryKind::Other` would not compile; Rust never gives `==` to a type for free.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EntryKind = "Missing" | "Folder" | "Track" | "Other";
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EntryKind {
    /// What: `Missing,` is the case with nothing at the path. A bare name with a comma is a case that carries no data.
    /// Why: a path the watcher named that is gone is a removal for the queue.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// "Missing"
    /// ```
    Missing,
    /// What: `Folder,` is the case of a folder.
    /// Why: a folder leads to more tracks, so the queue reads inside it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// "Folder"
    /// ```
    Folder,
    /// What: `Track,` is the case of an audio file the queue lists.
    /// Why: it is the only case that becomes a row.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// "Track"
    /// ```
    Track,
    /// What: `Other,` is the case of anything the queue does not list (a text file, a cover image, a socket).
    /// Why: such entries are ignored instead of being treated as errors.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// "Other"
    /// ```
    Other,
}

/// What: `pub(crate) trait Disk { kind, entries, walk }` declares an interface: a list of method signatures that any type
/// may implement. A `trait` is the closest thing Rust has to a TypeScript `interface`, and a type opts in with a separate
/// `impl Trait for Type` block.
/// Why: see the module description; the planner takes any `Disk`, so a test can pass an in-memory one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Disk { kind(path: string): EntryKind; entries(folder: string): [string, EntryKind][]; walk(folder: string): string[]; }
/// ```
pub(crate) trait Disk {
    /// What: `fn kind(&self, path: &Path) -> EntryKind;` is a method signature with no body (the `;` ends it). `&self` is a
    /// read-only borrow of the object the method is called on; `&Path` is a read-only borrow of the argument; `-> EntryKind`
    /// is the return type.
    /// Why: what a path is right now decides whether the queue removes, adds or looks at it again.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// kind(path: string): EntryKind;
    /// ```
    fn kind(&self, path: &Path) -> EntryKind;
    /// What: `fn entries(&self, folder: &Path) -> Vec<(PathBuf, EntryKind)>;` is a signature that answers a growable list of
    /// pairs. `Vec<T>` is an owned, growable list (siblings: `&[T]` is a borrowed view of one, and `[T; N]` is a fixed-size
    /// array); `(PathBuf, EntryKind)` is a tuple, a fixed pair like TypeScript's `[string, EntryKind]`.
    /// Why: a folder event is answered by the folder's own entries; a `Vec` is returned, not a slice, because the list is
    /// built here and the caller keeps it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// entries(folder: string): [string, EntryKind][];
    /// ```
    fn entries(&self, folder: &Path) -> Vec<(PathBuf, EntryKind)>;
    /// What: `fn walk(&self, folder: &Path) -> Vec<PathBuf>;` is a signature that answers an owned list of paths.
    /// Why: every track under a new or recreated folder, in queue order, so it can be compared with nothing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// walk(folder: string): string[];
    /// ```
    fn walk(&self, folder: &Path) -> Vec<PathBuf>;
}

/// What: `pub(crate) struct RealDisk;` declares a struct with no fields (a unit struct). It holds no data; it exists only
/// to carry the `Disk` methods for the real file system.
/// Why: the running player asks the real disk, with the folder walk's own rules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RealDisk implements Disk {} // no fields
/// ```
pub(crate) struct RealDisk;

/// What: `fn kind_of(path: &Path, file_type: std::fs::FileType) -> EntryKind` classifies one entry by its type, the way
/// the folder walk does. `std::fs::FileType` is the standard library's answer to "folder, file or symlink", spelled with
/// its full path because it was not imported. It is taken by value; it is a small value that the library lets you copy.
/// Why: a folder entry says whether it is a folder; a symlink needs one more look to see whether it leads to a file, and a
/// symlink to a folder is not followed, so the planner and the walk list the same tracks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kindOf = (path: string, type: FileType): EntryKind => type.isDir() ? "Folder" : isAudio(path) && ... ? "Track" : "Other";
/// ```
fn kind_of(path: &Path, file_type: std::fs::FileType) -> EntryKind {
    // What:     `if file_type.is_dir() { return EntryKind::Folder; }` checks the type and leaves early. `EntryKind::Folder`
    //           names one case of the enum; `::` joins the enum's name and the case.
    // Why:      A directory is a folder whatever its name looks like.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (fileType.isDirectory()) return "Folder";
    // ```
    if file_type.is_dir() {
        return EntryKind::Folder;
    }
    // What:     `if is_audio_file(path) && (file_type.is_file() || (file_type.is_symlink() && path.is_file())) { ... }` is one
    //           condition: the extension must be audio, and the entry must be a plain file, or a symlink that leads to a
    //           file (`path.is_file()` follows the link). `&&` is "and" and `||` is "or", as in TypeScript.
    // Why:      The folder walk lists a symlinked file but never follows a symlinked folder, and the planner must agree.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (isAudioFile(path) && (type.isFile() || (type.isSymbolicLink() && statSync(path).isFile()))) return "Track";
    // ```
    if is_audio_file(path) && (file_type.is_file() || (file_type.is_symlink() && path.is_file())) {
        return EntryKind::Track;
    }
    // What:     `return EntryKind::Other;` ends the function with the case for entries the queue does not list.
    // Why:      Everything that is not a folder and not a playable file falls here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return "Other";
    // ```
    return EntryKind::Other;
}

/// What: `impl Disk for RealDisk { ... }` says that `RealDisk` implements the `Disk` trait and holds the bodies of its three
/// methods. The signatures must match the trait's.
/// Why: this is the file-system answer the running player uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RealDisk implements Disk { kind(...) {...} entries(...) {...} walk(...) {...} }
/// ```
impl Disk for RealDisk {
    /// What: `fn kind(&self, path: &Path) -> EntryKind` does one `lstat` of the path (a file-system query that does not
    /// follow a symlink).
    /// Why: a missing path is a removal; a path that exists is looked at by its type.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// kind(path: string): EntryKind { try { return kindOf(path, lstatSync(path)); } catch (e) { return "Missing"; } }
    /// ```
    fn kind(&self, path: &Path) -> EntryKind {
        // What:     `return match std::fs::symlink_metadata(path) { ... };` asks the file system for the path's metadata and
        //           takes the answer apart. `symlink_metadata` answers a `Result`: `Ok(metadata)` on success and `Err(error)`
        //           on failure, Rust's stand-in for a call that may throw. `match` has one arm per case. The `;` after the
        //           closing brace ends the `return` statement.
        // Why:      The three arms separate "it exists", "it is not there" and "something else went wrong".
        // Gotcha:   `match` must cover every case; leaving out the final `Err(error)` arm would not compile.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // try { const metadata = lstatSync(path); return kindOf(path, metadata); }
        // catch (error) { if (error.code === "ENOENT") return "Missing"; logger.warn(...); return "Missing"; }
        // ```
        return match std::fs::symlink_metadata(path) {
            // What:     `Ok(metadata) => kind_of(path, metadata.file_type()),` is the success arm: it names the metadata
            //           and the arm's value is the classification of its file type. `.file_type()` reads the type out of
            //           the metadata.
            // Why:      A path that exists is classified by its type.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return kindOf(path, metadata.fileType());
            // ```
            Ok(metadata) => kind_of(path, metadata.file_type()),
            // What:     `Err(error) if error.kind() == std::io::ErrorKind::NotFound => EntryKind::Missing,` is a failure arm
            //           with a guard: it matches only when the `if` condition also holds. `error.kind()` is the category of
            //           the failure and `std::io::ErrorKind::NotFound` is the category "no such file".
            // Why:      A path that is gone is an ordinary answer ("Missing"), not an error worth a warning.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (error.code === "ENOENT") return "Missing";
            // ```
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => EntryKind::Missing,
            // What:     `Err(error) => { ... }` is the remaining failure arm: any other error (for example a permission
            //           problem). Its block logs a warning and its last expression, `EntryKind::Missing`, has no `;`, so it
            //           is the value of the arm.
            // Why:      The planner needs some answer, and a path that cannot be looked at is safest to treat as gone.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // logger.warn({ path, error }, "cannot look at a changed path; treated as missing"); return "Missing";
            // ```
            Err(error) => {
                // What:     `tracing::warn!(path = %path.display(), error = %error, "...");` writes a warning log line.
                //           `tracing::warn!` is a macro (the `!` marks code that expands at compile time). A field written
                //           `name = %value` adds a named field using the human-readable (`Display`) form; `.display()`
                //           gives a path such a form.
                // Why:      The repo logs every error path.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // logger.warn({ path, error }, "cannot look at a changed path; treated as missing");
                // ```
                tracing::warn!(path = %path.display(), error = %error, "cannot look at a changed path; treated as missing");
                // What:     `EntryKind::Missing` with no `;` is the arm's value: a block's last expression is what the
                //           block produces.
                // Why:      Answer "missing" for a path that cannot be looked at.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // return "Missing";
                // ```
                EntryKind::Missing
            }
        };
    }

    /// What: `fn entries(&self, folder: &Path) -> Vec<(PathBuf, EntryKind)>` makes one read of the folder and answers its
    /// direct folders and tracks as owned pairs.
    /// Why: a folder event is answered by the folder's own entries, not by a walk of what is below.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// entries(folder: string): [string, EntryKind][] { return readdirSync(folder).map(...).filter(...); }
    /// ```
    fn entries(&self, folder: &Path) -> Vec<(PathBuf, EntryKind)> {
        // What:     `let listing = match std::fs::read_dir(folder) { Ok(listing) => listing, Err(error) => { ...; return Vec::new(); } };`
        //           opens the folder for reading. `read_dir` answers a `Result`; the success arm hands the listing out (it
        //           becomes the value of the `match` and so of `listing`), and the failure arm logs and leaves the method
        //           with an empty list.
        // Why:      A folder that cannot be read has no entries to compare, and the refresh must go on without it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let listing; try { listing = opendirSync(folder); } catch (error) { logger.warn(...); return []; }
        // ```
        let listing = match std::fs::read_dir(folder) {
            Ok(listing) => listing,
            Err(error) => {
                // What:     `tracing::warn!(dir = %folder.display(), error = %error, "...");` is the same kind of warning
                //           log line as in `kind`, with the folder as the `dir` field.
                // Why:      The failure is logged, never swallowed.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // logger.warn({ dir: folder, error }, "cannot read a changed folder");
                // ```
                tracing::warn!(dir = %folder.display(), error = %error, "cannot read a changed folder");
                // What:     `return Vec::new();` leaves with a new empty `Vec`. `Vec::new()` is a constructor that makes an
                //           empty growable list and allocates nothing until the first push.
                // Why:      An unreadable folder reports no entries.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // return [];
                // ```
                return Vec::new();
            }
        };
        // What:     `return listing.flatten().filter_map(|entry| { ... }).collect();` is one chain of lazy steps. `.flatten()`
        //           turns the listing's items (each a `Result` that may have failed) into just the successful entries,
        //           dropping the failed ones. `.filter_map(|entry| ...)` runs the closure on each entry and keeps the
        //           `Some` results; `|entry| ...` is a closure, a function whose parameters sit between the bars.
        //           `.collect()` gathers the kept values into the method's return type, `Vec<(PathBuf, EntryKind)>`.
        // Why:      One pass over the folder produces exactly the pairs the planner needs, and an unreadable entry is
        //           skipped instead of stopping the pass.
        // Gotcha:   Nothing runs until `.collect()` pulls the values through; the steps before it only describe the work.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return readdirSync(folder, { withFileTypes: true }).flatMap((entry) => { ...; return kind !== "Other" ? [[path, kind]] : []; });
        // ```
        return listing
            .flatten()
            .filter_map(|entry| {
                // What:     `let file_type = entry.file_type().ok()?;` asks the entry for its type. `file_type()` answers a
                //           `Result`; `.ok()` converts it to an `Option` (`Some(value)` on success, `None` on failure,
                //           dropping the error); and the trailing `?` leaves the closure early with `None` when the
                //           `Option` is `None`, otherwise unwraps the value.
                // Why:      An entry whose type cannot be read is skipped, which in this `filter_map` means `None`.
                // Gotcha:   `?` is early return, not an optional-chaining dot as in TypeScript's `a?.b`.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const fileType = tryFileType(entry); if (fileType == null) return null;
                // ```
                let file_type = entry.file_type().ok()?;
                // What:     `let path: PathBuf = entry.path();` takes the entry's full path as an owned `PathBuf`.
                // Why:      The pair that is returned must own its path, because the entry is dropped at the end of the
                //           closure.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const path: string = entry.path;
                // ```
                let path: PathBuf = entry.path();
                // What:     `let kind: EntryKind = kind_of(&path, file_type);` classifies the entry. `&path` lends the path
                //           read-only, so `path` can still be moved into the returned pair afterwards.
                // Why:      The classification decides whether the entry is kept.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const kind: EntryKind = kindOf(path, fileType);
                // ```
                let kind: EntryKind = kind_of(&path, file_type);
                // What:     `return (kind != EntryKind::Other).then_some((path, kind));` turns a condition into an optional
                //           value. `bool.then_some(x)` answers `Some(x)` when the bool is true and `None` when it is false.
                //           `(path, kind)` is a tuple that moves `path` into the pair. `!=` is TypeScript's `!==`.
                // Why:      Entries the queue does not list are dropped; the rest become `(path, kind)` pairs.
                // Gotcha:   `then_some` builds its argument even when the bool is false (the pair is cheap); `.then(|| ...)`
                //           would defer it.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // return kind !== "Other" ? [path, kind] : null;
                // ```
                return (kind != EntryKind::Other).then_some((path, kind));
            })
            .collect();
    }

    /// What: `fn walk(&self, folder: &Path) -> Vec<PathBuf>` is the folder walk of one subtree.
    /// Why: a new or recreated folder has no queue entries to compare with, so everything in it is read once.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// walk(folder: string): string[] { return scanStreaming(folder, Infinity, () => {}); }
    /// ```
    fn walk(&self, folder: &Path) -> Vec<PathBuf> {
        // What:     `return crate::scan_stream::scan_streaming(folder, usize::MAX, &mut |_batch| {});` runs the same walk the
        //           first scan uses. `usize::MAX` is the largest value of `usize` (the unsigned integer that indexes
        //           memory; siblings: `u32`, `u64`), used as "never report a batch". `&mut |_batch| {}` is a closure that
        //           does nothing, lent mutably (`&mut`) because the function expects a callback it may call repeatedly.
        //           The `_` in `_batch` marks the parameter as deliberately unused. The path
        //           `crate::scan_stream::scan_streaming` is spelled in full because it was not imported.
        // Why:      Reusing the walk keeps the planner and the first scan listing exactly the same tracks; the empty
        //           callback is there because this caller wants no progress reports.
        // Gotcha:   `usize::MAX` is a plain number (the way `Infinity` is in TypeScript, but finite); a batch size that
        //           large simply never fills.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return scanStreaming(folder, Number.MAX_SAFE_INTEGER, () => {});
        // ```
        return crate::scan_stream::scan_streaming(folder, usize::MAX, &mut |_batch| {});
    }
}
