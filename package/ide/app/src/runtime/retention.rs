//! Remove the cache folders of other builds once they have gone unused for 30 days.
//!
//! What:
//!  Every build unpacks its parser libraries below `<cache>/monochromatic-ide/runtime/<key>/`,
//!       where `<key>` is a digest of its embedded files.
//!  [`mark_used`] renews `<key>/last-used`;
//!       [`remove_unused`] removes the other key folders whose last use lies more than
//!       [`UNUSED_LIMIT`] back.
//! Why:
//!  Each new build leaves a folder of about 33 MB behind.
//!  The user chose on 2026-10-06 to
//!      "Remove after N days unused",
//!  with 30 days.
//!
//! The rules:
//! - Last use is the modification time of `<key>/last-used`.
//!    The copy using that key renews it at
//!   every start and every parser load,
//!    so a copy that keeps running keeps its folder.
//!    A folder
//!   without the file (builds before this rule wrote none) is aged by its own modification time.
//! - The current key is never removed.
//! - Only direct children of the runtime folder are candidates,
//!    and only real folders (never
//!   symbolic links) named by 16 lowercase hexadecimal digits;
//!    anything else is left alone.
//!    When the
//!   runtime folder itself is a symbolic link,
//!    nothing is removed.
//! - A folder is first renamed to `.<key>.removing-<process>-<n>`,
//!    then removed with
//!   `fs::remove_dir_all`,
//!    which does not follow symbolic links:
//!    it removes a link itself,
//!    never what
//!   it points to (Rust standard library documentation of `remove_dir_all`).
//!    A removal cut short
//!   leaves only such a name,
//!    which the next start removes;
//!    the half-removed folder can never look
//!   recently used.
//! - Failures are logged and skipped:
//!    tidying the cache never stops the application.

/// The marker is written through the cache's private-file-and-rename step.
use super::cache::{replace_atomically, unpack};
/// Errors name the folder and the remedy.
use anyhow::Result;
/// Folders,
///  metadata without following links,
///  times,
///  and the removal counter.
use std::{
    fs,
    io::ErrorKind,
    os::unix::fs::DirBuilderExt,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant, SystemTime},
};

/// What:
///  How long a key folder may go unused before it is removed.
///  `Duration` is a span of time.
/// Why:
///  The user's choice of 2026-10-06:
///  30 days.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const UNUSED_LIMIT_MS = 30 * 24 * 60 * 60 * 1000;
/// ```
pub const UNUSED_LIMIT: Duration = Duration::from_secs(30 * 24 * 60 * 60);

/// What:
///  The file name whose modification time records a key folder's last use.
/// Why:
///  A folder's own time changes only when entries are added or removed,
///  not when a copy merely
///      reads a cached library,
///  so the copy renews this file itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MARKER = 'last-used';
/// ```
pub const MARKER: &str = "last-used";

/// What:
///  The marker's text,
///  for a person who finds the file.
/// Why:
///  Only the modification time matters;
///  the text explains what it is for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MARKER_TEXT = 'Renewed at every start and parser load ...';
/// ```
const MARKER_TEXT: &[u8] = b"Renewed by Monochromatic IDE at every start and parser load of the build with this key.\nOther key folders unused for 30 days are removed at start.\n";

/// What:
///  A process-wide counter for removal names,
///  incremented without a lock.
/// Why:
///  Two removals in one process must not pick the same temporary name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let nextRemoval = 0;
/// ```
static NEXT_REMOVAL: AtomicU64 = AtomicU64::new(0);

/// What:
///  What one pass over the runtime folder did,
///  by entry name.
///  `Vec<String>` is a growable list
///       of owned texts;
///  `#[derive(Default)]` lets `Sweep::default()` build one with empty lists.
/// Why:
///  Tests and the summary log line read it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Sweep = { removed: string[]; kept: string[]; ignored: string[] };
/// ```
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Sweep {
    /// Key folders removed,
    ///  and leftovers of removals cut short.
    pub removed: Vec<String>,
    /// Key folders left in place:
    ///  the current one,
    ///  recently used ones,
    ///  and ones that failed to go.
    pub kept: Vec<String>,
    /// Entries that are not key folders:
    ///  other names,
    ///  files,
    ///  and symbolic links.
    pub ignored: Vec<String>,
}

/// What:
///  Whether `name` has the shape of a key:
///  exactly 16 lowercase hexadecimal digits.
///       `bytes()` walks the text's bytes;
///  `matches!` tests a byte against the listed ranges.
/// Why:
///  Only folders this application created by that rule may be removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isKey(name: string): boolean { return /^[0-9a-f]{16}$/.test(name); }
/// ```
fn is_key(name: &str) -> bool {
    if name.len() != 16 {
        return false;
    }
    for byte in name.bytes() {
        if !matches!(byte, b'0'..=b'9' | b'a'..=b'f') {
            return false;
        }
    }
    return true;
}

/// What:
///  Whether `text` is made only of ASCII digits and dashes.
/// Why:
///  The suffix of a removal's temporary name is a process number and a counter joined by a dash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const allDigitsOrDashes = (text: string): boolean => /^[0-9-]*$/.test(text);
/// ```
fn all_digits_or_dashes(text: &str) -> bool {
    for byte in text.bytes() {
        if !(byte.is_ascii_digit() || byte == b'-') {
            return false;
        }
    }
    return true;
}

/// What:
///  Whether `name` is the temporary name of a removal:
///  `.<key>.removing-<digits and dashes>`.
///       `strip_prefix` returns the rest after a prefix,
///  or `None` without it.
/// Why:
///  A removal cut short leaves this name behind,
///  and nothing else creates it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isLeftover(name: string): boolean { return /^\.[0-9a-f]{16}\.removing-[0-9-]+$/.test(name); }
/// ```
fn is_leftover(name: &str) -> bool {
    // What: `let Some(rest) = ... else { return false; }` unpacks the text after the dot or leaves.
    // Why: Every other shape is not a leftover.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!name.startsWith('.')) return false; const rest = name.slice(1);
    // ```
    let Some(rest) = name.strip_prefix('.') else {
        return false;
    };
    // `split_once(".removing-")` divides the text at the first occurrence, giving a pair.
    let Some((key, suffix)) = rest.split_once(".removing-") else {
        return false;
    };
    return is_key(key) && !suffix.is_empty() && all_digits_or_dashes(suffix);
}

/// What:
///  The message that names the folder whose use could not be recorded and how to fix it.
/// Why:
///  Both steps of recording the use fail with the same advice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const useRecordRemedy = (keyFolder: string): string => `Cannot record the use of ${keyFolder}. Free disk space ...`;
/// ```
fn use_record_remedy(key_folder: &Path) -> String {
    return format!(
        "Cannot record the use of the language parser cache {}. Free disk space or fix its permissions, or point XDG_CACHE_HOME at a writable directory.",
        key_folder.display()
    );
}

/// What:
///  Create the key folder when needed (mode 0700) and write a fresh marker into it.
/// Why:
///  Called at start and before every parser load,
///  so the folder of a running copy never ages.
///      The marker is replaced through a new private file and a rename,
///  so a symbolic link planted
///      at the marker's name is replaced itself and nothing it points to is written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function markUsed(keyFolder: string): void // throws when the folder is not writable
/// ```
pub fn mark_used(key_folder: &Path) -> Result<()> {
    // What: `DirBuilder` creates the folder and missing parents with mode 0700.
    // Why: The same rule as the parser libraries: only the user may write here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // fs.mkdirSync(keyFolder, { recursive: true, mode: 0o700 });
    // ```
    if let Err(error) = fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(key_folder)
    {
        return Err(anyhow::Error::new(error).context(use_record_remedy(key_folder)));
    }
    if let Err(error) = replace_atomically(key_folder, &key_folder.join(MARKER), MARKER_TEXT) {
        return Err(anyhow::Error::new(error).context(use_record_remedy(key_folder)));
    }
    tracing::debug!(folder = %key_folder.display(), "language parser cache use recorded");
    return Ok(());
}

/// What:
///  Record the key folder's use,
///  then return the path of the cached library `name`,
///  writing
///       it first when absent or different (see [`unpack`]).
/// Why:
///  Every parser load renews the marker,
///  so a long-running copy that loads a parser keeps its
///      folder.
///  A marker that cannot be written is logged;
///  the unpack step reports the real problem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unpackMarked(keyFolder: string, name: string, bytes: Uint8Array): string
/// ```
pub fn unpack_marked(key_folder: &Path, name: &str, bytes: &[u8]) -> Result<PathBuf> {
    if let Err(error) = mark_used(key_folder) {
        tracing::warn!(folder = %key_folder.display(), error = %format!("{error:#}"), "language parser cache use not recorded");
    }
    return unpack(&key_folder.join("grammars"), name, bytes);
}

/// What:
///  The last use of a key folder:
///  its marker's modification time,
///  or the folder's own time
///       when there is no marker.
///  `symlink_metadata` reads an entry without following a link.
/// Why:
///  See the module rules;
///  a marker that is not a regular file is not trusted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lastUse(keyFolder: string): Date
/// ```
fn last_use(key_folder: &Path) -> std::io::Result<SystemTime> {
    let marker = key_folder.join(MARKER);
    // What: `match` on the marker's metadata: a regular file gives its time; a missing marker or one
    //       that is not a regular file falls through to the folder's own time.
    // Why: Builds before this rule wrote no marker.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const stat = tryLstat(marker); if (stat?.isFile()) return stat.mtime;
    // ```
    match fs::symlink_metadata(&marker) {
        Ok(metadata) if metadata.is_file() => return metadata.modified(),
        Ok(_) => {
            tracing::debug!(marker = %marker.display(), "language parser cache marker is not a regular file; using the folder's own time");
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            tracing::debug!(folder = %key_folder.display(), "language parser cache folder has no marker; using its own time");
        }
        Err(error) => return Err(error),
    }
    return fs::symlink_metadata(key_folder)?.modified();
}

/// What:
///  Rename `path` to a private temporary name in `runtime_folder`,
///  then remove it entirely.
/// Why:
///  See the module rules;
///  the rename makes a cut-short removal recognizable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removeFolder(runtimeFolder: string, name: string): void // throws on failure
/// ```
fn remove_folder(runtime_folder: &Path, name: &str) -> std::io::Result<()> {
    let number = NEXT_REMOVAL.fetch_add(1, Ordering::Relaxed);
    let doomed = runtime_folder.join(format!(".{name}.removing-{}-{number}", std::process::id()));
    fs::rename(runtime_folder.join(name), &doomed)?;
    return fs::remove_dir_all(&doomed);
}

/// What:
///  Decide one entry of the runtime folder and record the decision in `sweep`.
///  `&mut Sweep`
///       lends the record for changing.
/// Why:
///  Kept apart from the folder walk so each rule reads in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decide(entry: Dirent, runtimeFolder: string, currentKey: string, now: Date, limitMs: number, sweep: Sweep): void
/// ```
fn decide(
    entry: &fs::DirEntry,
    runtime_folder: &Path,
    current_key: &str,
    now: SystemTime,
    limit: Duration,
    sweep: &mut Sweep,
) {
    // What: `into_string()` gives the name as UTF-8 text, or `Err` with the raw name.
    // Why: A key is always ASCII; any other name is not this application's.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name = entry.name; // JS names are always text
    // ```
    let name = match entry.file_name().into_string() {
        Ok(text) => text,
        Err(raw) => {
            sweep.ignored.push(raw.to_string_lossy().into_owned());
            return;
        }
    };
    // What: `DirEntry::file_type` describes the entry itself; a symbolic link reports as a link,
    //       never as what it points to. `is_ok_and` is true only for a successful check that holds.
    // Why: Links are never followed or removed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const isFolder = entry.isDirectory(); // Dirent from withFileTypes does not follow links
    // ```
    let mut is_folder = false;
    if let Ok(kind) = entry.file_type() {
        is_folder = kind.is_dir();
    }
    if is_folder && is_leftover(&name) {
        let doomed = runtime_folder.join(&name);
        match fs::remove_dir_all(&doomed) {
            Ok(()) => {
                tracing::info!(folder = %doomed.display(), "removed the rest of a language parser cache removal that was cut short");
                sweep.removed.push(name);
            }
            Err(error) => {
                tracing::warn!(folder = %doomed.display(), %error, "rest of a cut-short language parser cache removal not removed");
                sweep.kept.push(name);
            }
        }
        return;
    }
    if !is_folder || !is_key(&name) {
        tracing::debug!(entry = %name, "runtime cache entry is not a key folder; left alone");
        sweep.ignored.push(name);
        return;
    }
    if name == current_key {
        sweep.kept.push(name);
        return;
    }
    let path = runtime_folder.join(&name);
    let last = match last_use(&path) {
        Ok(time) => time,
        Err(error) => {
            tracing::warn!(folder = %path.display(), %error, "language parser cache folder's last use unreadable; kept");
            sweep.kept.push(name);
            return;
        }
    };
    // What: `duration_since` fails for a time after `now`; `unwrap_or(Duration::ZERO)` treats that
    //       as used just now.
    // Why: A clock set back must never make a folder look old.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const unusedMs = Math.max(0, now.getTime() - last.getTime());
    // ```
    let unused = now.duration_since(last).unwrap_or(Duration::ZERO);
    let unused_days = unused.as_secs() / 86_400;
    if unused <= limit {
        tracing::debug!(folder = %path.display(), unused_days, "language parser cache folder of another build used recently; kept");
        sweep.kept.push(name);
        return;
    }
    match remove_folder(runtime_folder, &name) {
        Ok(()) => {
            tracing::info!(folder = %path.display(), unused_days, "removed the language parser cache folder of another build");
            sweep.removed.push(name);
        }
        Err(error) => {
            tracing::warn!(folder = %path.display(), unused_days, %error, "unused language parser cache folder not removed");
            sweep.kept.push(name);
        }
    }
}

/// What:
///  Remove every other key folder in `runtime_folder` unused for longer than `limit` at `now`.
///       Production passes `SystemTime::now()` and [`UNUSED_LIMIT`];
///  tests pass their own times.
/// Why:
///  The module rules;
///  the caller has already renewed the current key's marker.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removeUnused(runtimeFolder: string, currentKey: string, now: Date, limitMs: number): Sweep
/// ```
pub fn remove_unused(
    runtime_folder: &Path,
    current_key: &str,
    now: SystemTime,
    limit: Duration,
) -> Sweep {
    let started = Instant::now();
    let mut sweep = Sweep::default();
    // What: `match` on the runtime folder's own metadata, read without following a link.
    // Why: A runtime folder that is a symbolic link would lead outside it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const stat = lstat(runtimeFolder); if (!stat.isDirectory()) return sweep;
    // ```
    match fs::symlink_metadata(runtime_folder) {
        Ok(metadata) if metadata.is_dir() => {}
        Ok(_) => {
            tracing::warn!(folder = %runtime_folder.display(), "runtime cache folder is not a plain folder (it may be a symbolic link); no old key folder is removed");
            return sweep;
        }
        Err(error) => {
            tracing::debug!(folder = %runtime_folder.display(), %error, "runtime cache folder unreadable; nothing to remove");
            return sweep;
        }
    }
    let listing = match fs::read_dir(runtime_folder) {
        Ok(entries) => entries,
        Err(error) => {
            tracing::warn!(folder = %runtime_folder.display(), %error, "runtime cache folder not listed; no old key folder is removed");
            return sweep;
        }
    };
    for listed in listing {
        match listed {
            Ok(entry) => decide(&entry, runtime_folder, current_key, now, limit, &mut sweep),
            Err(error) => {
                tracing::warn!(folder = %runtime_folder.display(), %error, "runtime cache entry unreadable; skipped");
            }
        }
    }
    tracing::info!(
        folder = %runtime_folder.display(),
        removed = sweep.removed.len(),
        kept = sweep.kept.len(),
        ignored = sweep.ignored.len(),
        elapsed_us = started.elapsed().as_micros(),
        "runtime cache tidied"
    );
    return sweep;
}

/// Removal by age,
///  the current key,
///  links,
///  leftovers,
///  and marker renewal,
///  on disposable folders.
#[cfg(test)]
#[path = "retention_tests.rs"]
mod tests;
