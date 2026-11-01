//! Pure playback helpers with no audio device or threading:
//!  the per-sample
//! output stage (volume,
//!  headroom,
//!  clamp) and the path-to-file expansion the
//! queue needs.
//!  Kept apart from `engine`/`controller` so these can be unit
//! -tested directly and so each file stays within the line budget.

/// What:
///      `use std::path::{Path, PathBuf};`.
///  `Path` is a borrowed path view;
///           `PathBuf` is the owned,
///  growable version (like `&str` vs `String`).
/// Why:
///       The helpers take borrowed paths and return owned ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // both are just `string` in TS
/// ```
use std::path::{Path, PathBuf};

/// What:
///      `const AUDIO_EXTENSIONS: &[&str] = &[ ... ];`.
///  `&[&str]` is a BORROWED
///           slice (sibling:
///  the owned `Vec<&str>`) of borrowed string slices,
///  each
///           pointing at text baked into the binary.
///  The file extensions (lowercased,
///           no leading dot) this player treats as playable,
///  matching the documented
///           codec set:
///  FLAC,
///  WAV/PCM,
///  MP3,
///  Vorbis (Ogg),
///  Opus,
///  AAC-LC/ALAC (MP4),
///           and AIFF.
/// Why:
///       A folder holds more than music (cover art,
///  playlists,
///  and system files
///           like `.DS_Store` / `.nomedia` / `.database_uuid`);
///  this allowlist is the
///           single rule deciding what a scan enqueues,
///  so junk never reaches the queue.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const AUDIO_EXTENSIONS = ["flac", "wav", "wave", "mp3", "ogg", "oga", "opus",
///   "m4a", "m4b", "mp4", "aac", "aiff", "aif", "aifc"] as const;
/// ```
const AUDIO_EXTENSIONS: &[&str] = &[
    "flac", "wav", "wave", "mp3", "ogg", "oga", "opus", "m4a", "m4b", "mp4", "aac", "aiff", "aif",
    "aifc",
];

/// What:
///      `const APPLE_DOUBLE_PREFIX: &str = "._"`.
///  `&str` is a borrowed string
///           slice (sibling:
///  owned `String`) pointing at text baked into the binary.
///           It holds the two-character prefix Apple uses for AppleDouble resource-fork
///           sidecar files.
/// Why:
///       Naming the marker once keeps desktop scanning aligned with the Android sources.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const APPLE_DOUBLE_PREFIX = "._";
/// ```
const APPLE_DOUBLE_PREFIX: &str = "._";

/// What:
///      `fn is_apple_double_sidecar(path: &Path) -> bool`.
///  Private predicate that
///           checks the final filename for Apple's `._` sidecar prefix.
///  `&Path` is a
///           borrowed filesystem path;
///  `bool` is Rust's true/false type,
///  like TS `boolean`.
/// Why:
///       AppleDouble files often copy the real track's extension,
///  so extension filtering
///           alone would enqueue `._song.mp3` as if it were a real track.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isAppleDoubleSidecar(path: string): boolean { /* ...body below... *\/ }
/// ```
fn is_apple_double_sidecar(path: &Path) -> bool {
    // What:     `match path.file_name() { ... }`. `file_name()` returns `Option<&OsStr>`:
    //           `Some(name)` when the path has a final component, or `None` for paths like
    //           `/`. The `match` chooses a branch based on which wrapper is present.
    // Why:      Sidecar detection only makes sense on the final filename, not parent folders.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name = basename(path);
    // if (name === "") return false;
    // ```
    match path.file_name() {
        // What:     `Some(name) => name.to_string_lossy().starts_with(APPLE_DOUBLE_PREFIX)`.
        //           `Some(name)` unwraps the present filename. `.to_string_lossy()` converts
        //           OS text to UTF-8, replacing invalid bytes if needed; `.starts_with(...)`
        //           tests the named prefix.
        // Why:      ASCII `._` survives lossy conversion, so this catches AppleDouble sidecars
        //           even when the rest of the filename is not valid UTF-8.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return name.startsWith(APPLE_DOUBLE_PREFIX);
        // ```
        Some(name) => return name.to_string_lossy().starts_with(APPLE_DOUBLE_PREFIX),
        // What:     `None => false`. The path has no final filename component.
        // Why:      Without a filename, it cannot be an AppleDouble sidecar file.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return false;
        // ```
        None => return false,
    }
}

/// What:
///      `pub(crate) fn is_audio_file(path: &Path) -> bool`.
///  True when the path's
///           filename is not an AppleDouble sidecar and its extension is in
///           `AUDIO_EXTENSIONS`,
///  compared case-insensitively.
///  `&Path` is a borrowed path
///           (read-only).
///  `pub(crate)` so the session pruner reuses the same rule (visible
///           inside this crate but not outside it).
/// Why:
///       One predicate decides "does this belong in a music queue",
///  shared by the
///           folder scan and the session restore so they cannot disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isAudioFile(path: string): boolean {
///   const ext = extname(path).replace(/^\./, "").toLowerCase(); // "" when none
///   return AUDIO_EXTENSIONS.includes(ext);
/// }
/// ```
pub(crate) fn is_audio_file(path: &Path) -> bool {
    // What:     `if is_apple_double_sidecar(path) { return false; }` calls the sidecar
    //           predicate and exits early when the final filename starts with `._`.
    // Why:      AppleDouble sidecars copy the real track's extension, so the prefix guard must
    //           run before the extension allowlist.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (isAppleDoubleSidecar(path)) return false;
    // ```
    if is_apple_double_sidecar(path) {
        return false;
    }
    // What:     `match path.extension() { ... }`. `path.extension()` returns
    //           `Option<&OsStr>`: the part after the final dot, or `None` when there
    //           is none. A dotfile like `.DS_Store` has NO extension in Rust (the
    //           leading dot is not a separator), so it lands in the `None` arm.
    // Why:      Without an extension there is nothing to match against the allowlist.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const ext = extname(path); // "" when none
    // ```
    match path.extension() {
        // What:     `Some(ext) => AUDIO_EXTENSIONS.contains(&ext.to_string_lossy().to_lowercase().as_str())`.
        //           `ext.to_string_lossy()` makes a `Cow<str>` (replacing invalid
        //           bytes); `.to_lowercase()` returns an owned lowercased `String`;
        //           `.as_str()` borrows it as `&str`; the leading `&` makes the `&&str`
        //           that `slice.contains` compares against each entry.
        // Why:      Case-insensitive membership test, so `.FLAC` matches `flac`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return AUDIO_EXTENSIONS.includes(ext.toLowerCase());
        // ```
        Some(ext) => return AUDIO_EXTENSIONS.contains(&ext.to_string_lossy().to_lowercase().as_str()),
        // What:     `None => false`. No extension (extensionless name or a dotfile):
        //           not recognised as audio.
        // Why:      Skip extensionless and hidden files.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return false;
        // ```
        None => return false,
    }
}

/// What:
///      `pub(crate) fn process_sample(sample: f32, gain: f32) -> f32`.
///  The
///           per-sample output stage:
///  apply the combined gain (user volume times the
///           track's normalization gain),
///  then hard-clamp into the valid PCM range.
///           `pub(crate)` makes it visible to the controller module but not outside the
///           crate.
///  A plain free function (not a method) so it is unit-testable.
/// Why:
///       One spot defines exactly what reaches the ring buffer,
///  so the clamp guard
///           cannot be skipped and its behaviour can be tested directly.
///  Headroom now
///           comes from per-track true-peak normalization folded into `gain` (see the
///           `truepeak` and `measure` modules),
///  not a fixed factor here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function processSample(sample: number, gain: number): number {
///   return Math.max(-1, Math.min(1, sample * gain));
/// }
/// ```
pub(crate) fn process_sample(sample: f32, gain: f32) -> f32 {
    // What:     `(sample * gain).clamp(-1.0, 1.0)`. Multiply the raw sample by the
    //           combined gain, then `f32::clamp` pins the result into `-1.0..=1.0`
    //           (returns the bound when outside, else the value unchanged). No
    //           trailing `;`, so this tail expression is the return value. `-1.0`/`1.0`
    //           are the valid f32 PCM range PipeWire expects.
    // Why:      Final guard against any sample leaving the legal range; with true-peak
    //           normalization the clamp rarely fires, but it backstops measurement
    //           error and any residual decoder overshoot.
    // Gotcha:   `clamp` PANICS only if a BOUND is NaN; ours are constants, so it never
    //           panics. A NaN product passes through as NaN; decoders do not emit NaN.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return Math.max(-1, Math.min(1, sample * gain));
    // ```
    return (sample * gain).clamp(-1.0, 1.0)
}

/// What:
///      `pub(crate) fn frames_to_secs(frames: u64, rate: u32) -> f64`.
///  Convert a
///           frame count to seconds at a given sample rate,
///  returning `0.0` when the
///           rate is unknown (`0`).
///  `f64` (sibling:
///  `f32`) is the seconds-as-f64 time
///           contract shared across threads;
///  `u64`/`u32` are unsigned counts.
/// Why:
///       Both the session snapshot and the position throttle need frames -> secs;
///           one helper keeps the divide-by-zero guard in a single place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function framesToSecs(frames: number, rate: number): number {
///   if (rate === 0) return 0;
///   return frames / rate;
/// }
/// ```
pub(crate) fn frames_to_secs(frames: u64, rate: u32) -> f64 {
    // What:     `if rate == 0 { return 0.0; }`. Early return guarding the divide.
    // Why:      An unknown rate has no meaningful position; avoid dividing by zero.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (rate === 0) return 0;
    // ```
    if rate == 0 {
        return 0.0;
    }
    // What:     `frames as f64 / rate as f64`. `as f64` widens both integers to float,
    //           then divide. Tail expression -> return.
    // Why:      seconds = frames / rate.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return frames / rate;
    // ```
    return frames as f64 / rate as f64
}

/// What:
///      `pub(crate) fn expand_paths(paths: Vec<PathBuf>) -> Vec<PathBuf>`.
///  Turn
///           the opened paths into a flat file list:
///  a directory expands to every file
///           under it,
///  RECURSIVELY (subfolders included);
///  a plain path passes through
///           unchanged.
///  Takes the vector BY VALUE (owned) so it can move each path out.
///           `pub(crate)` so the controller can call it.
/// Why:
///       The queue holds files,
///  but the UI opens a folder,
///  which should enqueue all
///           of its tracks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function expandPaths(paths: string[]): string[] {
///   const out: string[] = [];
///   for (const path of paths) isDir(path) ? out.push(...collectDirFiles(path)) : out.push(path);
///   return out;
/// }
/// ```
pub(crate) fn expand_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    // What:     `let mut out: Vec<PathBuf> = Vec::new();`. The accumulating result.
    //           `mut` because we push into it; explicit type because it starts empty.
    // Why:      Collect the expanded files.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const out: string[] = [];
    // ```
    let mut out: Vec<PathBuf> = Vec::new();

    // What:     `for path in paths { ... }`. Consume each opened path BY VALUE (the
    //           loop takes ownership of each element out of `paths`).
    // Why:      Classify file vs directory.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const path of paths) { ... }
    // ```
    for path in paths {
        // What:     `if path.is_dir() { ... } else { ... }`. `is_dir()` checks the
        //           filesystem (following symlinks).
        // Why:      Directories need recursive expansion.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (isDir(path)) { ... } else { ... }
        // ```
        if path.is_dir() {
            // What:     `out.extend(collect_dir_files(&path));`. `&path` lends the
            //           directory; `collect_dir_files` returns its files; `.extend(...)`
            //           MOVES each returned path into `out`.
            // Why:      Recursively enqueue the folder's tracks.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // out.push(...collectDirFiles(path));
            // ```
            out.extend(collect_dir_files(&path));
        } else {
            // What:     `if is_apple_double_sidecar(&path) { continue; }` borrows `path` with
            //           `&path`, checks whether it is an AppleDouble sidecar, and skips this
            //           loop iteration when it is.
            // Why:      Directly opened `._song.mp3` is still junk, even though direct non-audio
            //           files continue through to the decoder's normal error path.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (isAppleDoubleSidecar(path)) continue;
            // ```
            if is_apple_double_sidecar(&path) {
                continue;
            }
            // What:     `out.push(path);`. A plain non-sidecar path: keep it as-is (MOVES it
            //           into `out`).
            // Why:      Could be a file (or non-existent; the decoder will report), but Apple's
            //           `._` sidecars are never meaningful opened tracks.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // out.push(path);
            // ```
            out.push(path);
        }
    }

    // What:     `out`. Tail expression -> the expanded list is returned.
    // Why:      Hand back the flat file list.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return out;
    // ```
    return out
}

/// What:
///      `fn collect_dir_files(root: &Path) -> Vec<PathBuf>`.
///  Walk a directory tree
///           and return every file under it,
///  sorted within each folder,
///  with a folder's
///           own files listed before its subfolders' files.
///  `&Path` is a borrowed path
///           (we only read it).
///  Private:
///  only `expand_paths` calls it.
/// Why:
///       Opening a folder should enqueue all its tracks,
///  including nested ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function collectDirFiles(root: string): string[] {
///   const out: string[] = []; const stack = [root];
///   while (stack.length) { /* read dir, sort, push subdirs reversed */ }
///   return out;
/// }
/// ```
fn collect_dir_files(root: &Path) -> Vec<PathBuf> {
    // What:     `crate::scan_stream::scan_streaming(root, usize::MAX, &mut |_batch| {})`. The one folder walk of the
    //           app, called with a report callback that ignores its batches.
    // Why:      The streaming walk (which also reports batches, for Search while the scan runs) and this plain walk
    //           must list the same tracks in the same order, so they share one implementation.
    return crate::scan_stream::scan_streaming(root, usize::MAX, &mut |_batch| {});
}

/// What:
///      `pub(crate) fn file_name_of(path: &Path) -> String`.
///  The display filename
///           of a path (final component),
///  or the whole path if it has none.
///  `pub(crate)`
///           so the controller can call it.
/// Why:
///       Filename-only metadata policy for the UI.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fileNameOf(path: string): string { return basename(path) || String(path); }
/// ```
pub(crate) fn file_name_of(path: &Path) -> String {
    // What:     `match path.file_name() { ... }`. `file_name()` returns `Option<&OsStr>`
    //           (the last component), or `None` (e.g. `/`).
    // Why:      Extract the filename.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name = basename(path);
    // ```
    match path.file_name() {
        // What:     `Some(name) => name.to_string_lossy().into_owned()`.
        //           `to_string_lossy()` converts the OS string to a `Cow<str>`
        //           (replacing invalid bytes); `.into_owned()` makes an owned `String`.
        // Why:      Need an owned UTF-8 string for the UI.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return name;
        // ```
        Some(name) => return name.to_string_lossy().into_owned(),
        // What:     `None => path.display().to_string()`. Fall back to the full path
        //           text. `display()` formats the path; `.to_string()` owns it.
        // Why:      Always show something.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return String(path);
        // ```
        None => return path.display().to_string(),
    }
}

/// What:
///      `#[cfg(test)] #[path = "playback_tests.rs"] mod tests;` declares a
///           test-only submodule whose code lives in the sibling file
///           `playback_tests.rs`.
///  `#[cfg(test)]` gates it to test builds only;
///           `#[path = "..."]` aims the module at a flat sibling file instead of the
///           default `playback/tests.rs` subdirectory lookup.
///  The file stays the
///           `tests` CHILD of playback,
///  so its `use super::*` reaches the module items
///           (including private ones) unchanged.
/// Why:
///       Keep `playback.rs` to production code;
///  the tests live beside it without
///           inflating this file or its max-lines budget (sibling `*_tests.rs` files
///           are exempt from the linter).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // playback.unit.test.ts, run only by the test runner
/// ```
#[cfg(test)]
#[path = "playback_tests.rs"]
mod tests;
