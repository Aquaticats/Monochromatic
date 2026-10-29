//! Read authoritative disk text without changing the displayed document or filesystem.

/// Helix correspondence is prepared separately from applying current reading state.
use crate::document::{Document, Reload};
/// I/O failures retain their affected input and operation context.
use anyhow::{Result, bail};
/// What:
///  `Read` provides `read_to_string` on an open `File`;
///  `Path` borrows a filesystem name,
///  unlike
///       the owned `PathBuf` sibling;
///  `Duration` is a time span and `SystemTime` a wall-clock time,
///       the clock file modification times use (`Instant`,
///  the monotonic sibling,
///  has no file times).
/// Why:
///  A read operation need not copy or retain the caller's path,
///  and the quiet check compares the
///      open file's modification time with the wall-clock time the read began.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readReload(snapshot: Document, path: string): Reload | undefined;
/// ```
use std::{
    fs::File,
    io::Read,
    path::Path,
    time::{Duration, SystemTime},
};

/// Outcome of a read that is accepted only when the file has been quiet for a while.
///
/// What:
///  an `enum` whose `Read` variant carries the usual read result and whose `RecentlyWritten`
///       variant carries nothing,
///  like a TS union `{ kind: 'read'; reload?: Reload } | { kind: 'recent' }`.
/// Why:
///  A read that no change notification asked for (the safety sweep,
///  a timer,
///  a highlighting retry)
///      can meet a save in progress,
///  and its bytes must then not replace the displayed text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type QuietRead = { kind: 'read'; reload: Reload | undefined } | { kind: 'recentlyWritten' };
/// ```
pub enum QuietRead {
    /// The file was last modified at least the quiet period before the read began,
    ///  so the bytes
    /// are a finished state.
    Read(
        /// The prepared change,
        ///  or `None` when the bytes equal the displayed text.
        Option<Reload>,
    ),
    /// The file was modified within the quiet period before the read began,
    ///  or during the read,
    /// or its modification time lies in the future:
    ///  a save may be in progress,
    ///  so the bytes are dropped.
    RecentlyWritten,
}

/// Open a regular file,
///  read it as UTF-8,
///  and return the text with the still-open handle.
fn read_regular(path: &Path) -> Result<(String, File)> {
    // What: each failure returns early with a message that names the affected path.
    // Why: A read failure must not silently replace visible source with empty text.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const metadata = await stat(path); // throws with path context
    // ```
    let metadata = match std::fs::metadata(path) {
        Ok(found) => found,
        Err(error) => {
            return Err(anyhow::Error::new(error)
                .context(format!("Cannot inspect source file {}", path.display())));
        }
    };
    if !metadata.is_file() {
        // bail! returns an error rather than opening a directory or potentially blocking device.
        bail!(
            "Cannot refresh {}: source is not a regular file",
            path.display()
        );
    }
    let mut file = match File::open(path) {
        Ok(found) => found,
        Err(error) => {
            return Err(anyhow::Error::new(error)
                .context(format!("Cannot open source file {}", path.display())));
        }
    };
    // What: `String::new()` makes an empty owned string (the borrowed sibling `&str` cannot grow);
    //       `&mut source` lends it to `read_to_string`, which appends the whole file to it.
    // Why: Reading through the open handle lets the caller ask that same file for its modification time.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let source = ''; source += readFileSync(handle, 'utf8');
    // ```
    let mut source = String::new();
    if let Err(error) = file.read_to_string(&mut source) {
        return Err(anyhow::Error::new(error).context(format!(
            "Cannot read source file {} as UTF-8",
            path.display()
        )));
    }
    // What: a tuple `(source, file)` returned inside `Ok`, like a TS array `[source, file]`.
    // Why: The quiet check needs the open handle after the read; the plain read just drops it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return [source, handle];
    // ```
    return Ok((source, file));
}

/// Prepare correspondence only if the bytes differ from the displayed text.
fn compare(snapshot: &Document, path: &Path, source: &str) -> Option<Reload> {
    // What: Iterator::eq compares bytes incrementally without cloning the old rope into a string.
    // Why: An unchanged poll must not increment the source revision or compute a diff.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (sameBytes(snapshot.text, source)) return undefined;
    // ```
    if snapshot.text().bytes().eq(source.bytes()) {
        // None is a successful unchanged read, not a suppressed read failure.
        return None;
    }
    tracing::debug!(path = %path.display(), base = snapshot.revision(), "preparing changed disk source");
    // What: Some carries the prepared replacement on success.
    // Why: Only the UI thread applies this result to the latest selection.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return snapshot.prepareReload(source);
    // ```
    return Some(snapshot.prepare_reload(source));
}

/// Read one regular UTF-8 file and prepare correspondence only if its bytes changed.
/// The caller retains the previous document on missing-file or decoding failures.
pub fn read_reload(snapshot: &Document, path: &Path) -> Result<Option<Reload>> {
    // What: `let (source, _file) = ...?` takes the tuple apart; `_file` is the handle, closed when dropped.
    // Why: The plain read has no use for the handle after reading.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [source] = readRegular(path);
    // ```
    let (source, _file) = read_regular(path)?;
    return Ok(compare(snapshot, path, &source));
}

/// What:
///  The last modification time of an open file.
/// Why:
///  The open handle is the file whose bytes were read,
///  and both steps fail with an I/O error the caller names with the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const modificationTime = (handle: number): Date => fstatSync(handle).mtime;
/// ```
fn modification_time(file: &File) -> std::io::Result<SystemTime> {
    let attributes = file.metadata()?;
    return attributes.modified();
}

/// Read like `read_reload`,
///  but drop the bytes when the open file was modified less than `quiet` before
/// the read began,
///  or during it.
///  A save in progress keeps writing and moving the modification time,
/// so its truncated or half-written state is never returned,
///  whatever order notifications arrive in.
/// A writer that pauses longer than `quiet` mid-save is read like any finished file.
pub fn read_reload_if_quiet(
    snapshot: &Document,
    path: &Path,
    quiet: Duration,
) -> Result<QuietRead> {
    let started = SystemTime::now();
    let (source, file) = read_regular(path)?;
    // What: `metadata()` asks the open file (not the path) for its attributes; `modified()` returns its
    //       last modification time as a `Result`, and `?` returns an error on systems without one.
    // Why: The open handle is the file whose bytes were read, even if another file was renamed into place.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const modified = fstatSync(handle).mtime;
    // ```
    let modified = match modification_time(&file) {
        Ok(found) => found,
        Err(error) => {
            return Err(anyhow::Error::new(error).context(format!(
                "Cannot read the modification time of source file {}",
                path.display()
            )));
        }
    };
    // What: `duration_since` returns `Ok(age)` when `modified` is not after `started`, and `Err` when it is
    //       (a write during the read, or a modification time from a clock ahead of this one).
    // Why: Only a file untouched for the whole quiet period before the read is a finished save.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const age = started - modified; if (age < 0 || age < quiet) return { kind: 'recentlyWritten' };
    // ```
    let quiet_enough = match started.duration_since(modified) {
        Ok(age) => age >= quiet,
        Err(ahead) => {
            tracing::debug!(path = %path.display(), ahead = ?ahead.duration(), "source modification time is after the read began");
            false
        }
    };
    if !quiet_enough {
        tracing::debug!(path = %path.display(), "dropped a timer read of a source written within the quiet period");
        return Ok(QuietRead::RecentlyWritten);
    }
    return Ok(QuietRead::Read(compare(snapshot, path, &source)));
}
