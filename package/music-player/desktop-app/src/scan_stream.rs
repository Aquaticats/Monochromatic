//! Folder walk that reports the tracks it finds while it is still walking.
//!
//! What: `scan_streaming` walks a root the way `expand_paths` always has (files of a folder in sorted order, then its
//! subfolders in sorted order) and calls `on_batch` with each run of newly found paths, so the caller can use them
//! before the walk ends. The list it returns is the whole walk, in the same order.
//! Why: D111 requires Search to work while the library is still being scanned. The walk of a large library takes
//! seconds, and a path's position in the walk is its position in the queue, so the batches can be indexed at once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const all = scanStreaming(root, 5000, (batch) => index.append(batch));
//! ```

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::natural_order::natural_cmp;
use crate::playback::is_audio_file;

/// What:
///      `pub(crate) const STREAM_BATCH: usize = 5_000`.
///  How many newly found tracks make a batch.
/// Why:
///       Five thousand paths normalize in about fifteen milliseconds, small enough that no step of the feed is long.
pub(crate) const STREAM_BATCH: usize = 5_000;

/// What:
///      `pub(crate) const FIRST_BATCH_AFTER: Duration`.
///  How long the walk waits before it reports a first batch smaller than `STREAM_BATCH`.
/// Why:
///       A library whose first folders are small should still become searchable quickly.
pub(crate) const FIRST_BATCH_AFTER: Duration = Duration::from_millis(150);

/// What:
///      `pub(crate) const SCAN_PAUSE_ENV: &str = "MUSIC_PLAYER_SCAN_PAUSE_MS"`.
///  The environment variable that makes the folder scan deliberately slow.
/// Why:
///       D111 is tested with a scan that is slow on purpose: the walk of a large library takes seconds, which is too
///  short to type a query in, so a test or an end-to-end run sets this to pause after each reported batch.
pub(crate) const SCAN_PAUSE_ENV: &str = "MUSIC_PLAYER_SCAN_PAUSE_MS";

/// What:
///      `pub(crate) fn pause_from_environment() -> Duration`.
///  The pause to take after each reported batch: the value of `MUSIC_PLAYER_SCAN_PAUSE_MS` in milliseconds, or zero.
/// Why:
///       See `SCAN_PAUSE_ENV`; unset or unparsable means no pause, so normal runs are never slowed.
pub(crate) fn pause_from_environment() -> Duration {
    return parse_pause(std::env::var(SCAN_PAUSE_ENV).ok().as_deref());
}

/// What:
///      `fn parse_pause(text: Option<&str>) -> Duration`.
///  The pause a variable value means: whole milliseconds, or zero for a missing or unparsable value.
/// Why:
///       Kept apart from the environment read so the rule can be tested without touching the process environment.
fn parse_pause(text: Option<&str>) -> Duration {
    let milliseconds: u64 = text.and_then(|value| return value.trim().parse::<u64>().ok()).unwrap_or(0);
    return Duration::from_millis(milliseconds);
}

/// What:
///      `fn sort_naturally(paths: &mut Vec<PathBuf>)`.
///  Sorts sibling paths by file name in natural order (`Track 2` before `Track 10`, case ignored), with the whole
///  path as the final tie-break.
/// Why:
///       The walk order is the queue order, so this one sort decides how rows are listed and in which order they play
///  (LS5). Sibling paths share their parent, so the file name is the part that differs. The name is read once per path,
///  not once per comparison, because a flat folder can hold tens of thousands of files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// paths.sort((a, b) => naturalCmp(basename(a), basename(b)) || (a < b ? -1 : 1));
/// ```
fn sort_naturally(paths: &mut Vec<PathBuf>) {
    let mut keyed: Vec<(String, PathBuf)> = std::mem::take(paths)
        .into_iter()
        .map(|path| {
            let name: String = path.file_name().map_or_else(String::new, |name| return name.to_string_lossy().into_owned());
            return (name, path);
        })
        .collect();
    keyed.sort_by(|left, right| return natural_cmp(&left.0, &right.0).then_with(|| return left.1.cmp(&right.1)));
    paths.extend(keyed.into_iter().map(|(_name, path)| return path));
}

/// What:
///      `pub(crate) fn scan_streaming(root: &Path, batch: usize, on_batch: &mut dyn FnMut(&[PathBuf])) -> Vec<PathBuf>`.
///  Walks `root` and returns every audio file in queue order, reporting new runs of at least `batch` paths on the way.
/// Why:
///       See the module description. Pass `usize::MAX` for no reports (a final run may still be reported at the end).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scanStreaming(root: string, batch: number, onBatch: (paths: string[]) => void): string[] { /* DFS */ }
/// ```
pub(crate) fn scan_streaming(root: &Path, batch: usize, on_batch: &mut dyn FnMut(&[PathBuf])) -> Vec<PathBuf> {
    let started: Instant = Instant::now();
    tracing::info!(root = %root.display(), "folder walk started");
    let mut out: Vec<PathBuf> = Vec::new();
    let mut reported: usize = 0;
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(error) => {
                tracing::warn!(dir = %dir.display(), error = %error, "cannot read dir");
                continue;
            }
        };
        let mut files: Vec<PathBuf> = Vec::new();
        let mut subdirs: Vec<PathBuf> = Vec::new();
        for entry in entries.flatten() {
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(_) => continue,
            };
            let path: PathBuf = entry.path();
            // A directory entry already says whether it is a regular file, so only a symlink needs the extra `stat`
            // that `Path::is_file` makes; on a cold disk cache that call cost about a hundred microseconds per file.
            if file_type.is_dir() {
                subdirs.push(path);
            } else if is_audio_file(&path) && (file_type.is_file() || (file_type.is_symlink() && path.is_file())) {
                files.push(path);
            }
        }
        sort_naturally(&mut files);
        sort_naturally(&mut subdirs);
        out.extend(files);
        stack.extend(subdirs.into_iter().rev());
        let waiting: usize = out.len() - reported;
        let due: bool = waiting >= batch || (reported == 0 && waiting > 0 && started.elapsed() >= FIRST_BATCH_AFTER);
        if due {
            tracing::debug!(first = reported, found = out.len() - reported, elapsed_ms = started.elapsed().as_millis(), "folder walk reports a batch");
            on_batch(&out[reported..]);
            reported = out.len();
        }
    }
    if reported < out.len() {
        on_batch(&out[reported..]);
    }
    tracing::info!(tracks = out.len(), elapsed_ms = started.elapsed().as_millis(), "folder walk finished");
    return out;
}

/// What:
///      `pub(crate) fn batch_names(root: &Path, paths: &[PathBuf]) -> Vec<String>`.
///  The names of newly found paths, relative to the scan root and joined with `/`.
/// Why:
///       Search needs display names before the whole queue exists. For a library with more than one top-level entry
///  these equal the queue's display names; the final queue replaces them if they differ.
pub(crate) fn batch_names(root: &Path, paths: &[PathBuf]) -> Vec<String> {
    return paths
        .iter()
        .map(|path| {
            let relative: &Path = path.strip_prefix(root).unwrap_or(path);
            return relative
                .components()
                .map(|component| return component.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
        })
        .collect();
}

/// Unit tests in `scan_stream_tests.rs`,
///  pulled in at this point.
#[cfg(test)]
#[path = "scan_stream_tests.rs"]
mod tests;
