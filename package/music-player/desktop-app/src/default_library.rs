//! The folder the player opens when no session names one: the user's music folder.
//!
//! What: `first_existing_dir` picks the first candidate that is a folder, and `home_music_folder` names the `Music`
//! folder inside a home folder.
//! Why: D95 says a library is always open, but the app looked for the music folder only in `XDG_MUSIC_DIR`, the
//! user-dirs file and the `xdg-user-dir` tool. A home folder that has a `Music` folder and none of those showed the
//! "No folder is open" screen (the polish review finding EM7). `Music` inside the home folder is the last candidate.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const musicDir = [env.XDG_MUSIC_DIR, userDirs.audio, xdgUserDir("MUSIC"), join(homedir(), "Music")].find(isDir);
//! ```

use std::path::{Path, PathBuf};

/// What:
///      `pub fn home_music_folder(home: &Path) -> PathBuf`.
///  The `Music` folder inside `home`.
/// Why:
///       The last place to look: Linux file managers, macOS and Windows all create or expect it, and a person who keeps
///  music in the home folder keeps it there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const homeMusicFolder = (home: string) => join(home, "Music");
/// ```
pub fn home_music_folder(home: &Path) -> PathBuf {
    return home.join("Music");
}

/// What:
///      `pub fn first_existing_dir<I: Iterator<Item = Option<PathBuf>>>(candidates: I) -> Option<PathBuf>`.
///  The first candidate that exists as a folder, or `None`.
/// Why:
///       The candidates are produced one at a time, so the slow `xdg-user-dir` tool runs only when the earlier places
///  gave nothing. A candidate that names a missing folder is skipped, not an answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function firstExistingDir(candidates: Iterable<string | null>): string | null { for (const c of candidates) if (c && isDir(c)) return c; return null; }
/// ```
pub fn first_existing_dir<I: Iterator<Item = Option<PathBuf>>>(candidates: I) -> Option<PathBuf> {
    for (position, candidate) in candidates.enumerate() {
        let Some(path) = candidate else {
            tracing::debug!(position, "music folder candidate: not set");
            continue;
        };
        if path.is_dir() {
            tracing::info!(position, path = %path.display(), "music folder candidate accepted");
            return Some(path);
        }
        tracing::debug!(position, path = %path.display(), "music folder candidate: not a folder");
    }
    tracing::info!("no music folder candidate is a folder");
    return None;
}

#[cfg(test)]
#[path = "default_library_tests.rs"]
mod tests;
