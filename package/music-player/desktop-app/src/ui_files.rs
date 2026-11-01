//! Platform file actions behind the track menu: show in the file manager, copy the file name,
//! move to the trash, and restore from the trash for Undo.
//!
//! What: each function performs one OS action and reports a `FileActionError` on failure.
//! Why: the menu decides which action runs; this module only knows how each OS does it.

use std::fmt;
use std::path::{Path, PathBuf};

/// What: `FileActionError` is the failure of one file action, with the path or text it concerned.
/// Why: the caller reports the failure as feedback, so each variant carries the detail to show.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FileActionError = { kind: "unsupported" | "spawn" | "clipboard" | "trash" | "restore"; detail: string };
/// ```
#[derive(Debug)]
pub enum FileActionError {
    /// The path has no parent folder to open.
    NoParent(PathBuf),
    /// A file manager process could not be started.
    Spawn(String),
    /// The clipboard could not be written.
    Clipboard(String),
    /// The file could not be moved to the trash.
    Trash(String),
    /// The file could not be restored from the trash; the text is the whole plain sentence to show.
    Restore(String),
}

impl fmt::Display for FileActionError {
    /// What: `fmt` writes the failure in plain words for the feedback message.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            Self::NoParent(path) => write!(formatter, "{} has no folder to open", path.display()),
            Self::Spawn(detail) => write!(formatter, "The file manager could not be started: {detail}"),
            Self::Clipboard(detail) => write!(formatter, "The clipboard could not be written: {detail}"),
            Self::Trash(detail) => write!(formatter, "The file could not be moved to the trash: {detail}"),
            Self::Restore(sentence) => formatter.write_str(sentence),
        };
    }
}

impl std::error::Error for FileActionError {}

/// What: `show_in_file_manager` opens the folder holding `path` with the file selected where the OS allows it.
/// Why: Linux has no select-file route, so it opens the parent; macOS and Windows select the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function showInFileManager(path: string): void { ... };
/// ```
#[cfg(target_os = "linux")]
pub fn show_in_file_manager(path: &Path) -> Result<(), FileActionError> {
    let parent = path.parent().ok_or_else(|| return FileActionError::NoParent(path.to_path_buf()))?;
    return spawn("xdg-open", |command| {
        command.arg(parent);
    });
}

/// What: `show_in_file_manager` on macOS opens Finder with the file selected (`open -R`).
#[cfg(target_os = "macos")]
pub fn show_in_file_manager(path: &Path) -> Result<(), FileActionError> {
    return spawn("open", |command| {
        command.arg("-R").arg(path);
    });
}

/// What: `show_in_file_manager` on Windows opens Explorer with the file selected (`explorer /select,`).
#[cfg(windows)]
pub fn show_in_file_manager(path: &Path) -> Result<(), FileActionError> {
    return spawn("explorer", |command| {
        command.arg(format!("/select,{}", path.display()));
    });
}

/// What: `show_in_file_manager` on other platforms reports that the action is unavailable.
#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
pub fn show_in_file_manager(_path: &Path) -> Result<(), FileActionError> {
    return Err(FileActionError::Spawn("no file manager is known for this platform".to_string()));
}

/// What: `spawn` starts a program with arguments set by `configure` and does not wait for it.
/// Why: a file manager stays open after the player continues, so the player must not block on it.
fn spawn(program: &str, configure: impl FnOnce(&mut std::process::Command)) -> Result<(), FileActionError> {
    let mut command = std::process::Command::new(program);
    configure(&mut command);
    return command
        .spawn()
        .map(|_child| ())
        .map_err(|error| return FileActionError::Spawn(format!("{program}: {error}")));
}

/// What: `copy_text` writes `text` to the system clipboard.
/// Why: Copy filename puts the file name on the clipboard, and a failed write must be reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function copyText(text: string): Promise<void> { await navigator.clipboard.writeText(text); }
/// ```
pub fn copy_text(text: &str) -> Result<(), FileActionError> {
    let mut clipboard = arboard::Clipboard::new().map_err(|error| return FileActionError::Clipboard(error.to_string()))?;
    return clipboard
        .set_text(text)
        .map_err(|error| return FileActionError::Clipboard(error.to_string()));
}

/// What: `move_to_trash` moves `path` to the OS trash.
/// Why: D8 deletes immediately and offers Undo, so the trash is the recoverable place for the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function moveToTrash(path: string): void { ... };
/// ```
pub fn move_to_trash(path: &Path) -> Result<(), FileActionError> {
    trash::delete(path).map_err(|error| return FileActionError::Trash(error.to_string()))?;
    remember_trashed(path);
    return Ok(());
}

/// What: `remember_trashed` records that this player just moved `path` to the trash.
/// Why: see `was_just_trashed`; kept apart from the trash call so a test can record a path without touching the real trash.
fn remember_trashed(path: &Path) {
    if let Ok(mut trashed) = TRASHED.lock() {
        trashed.push((path.to_path_buf(), std::time::Instant::now()));
    }
}

/// What: `comparable_path` is a path with its folder resolved through symlinks and its file name kept.
/// Why: a trashed file no longer exists, so the file itself cannot be resolved, but its folder can. The trash records the
/// folder as the system resolved it, while the player may have opened the library through a symlinked path (the polish
/// review finding PB3: Undo failed with "is not in the trash" for a library opened through a symlink).
/// In TS you'd write (pseudocode):
/// ```ts
/// const comparablePath = (p: string) => join(realpathSync(dirname(p)), basename(p));
/// ```
pub fn comparable_path(path: &Path) -> Option<PathBuf> {
    let name = path.file_name()?;
    let parent = path.parent()?;
    let resolved = match std::fs::canonicalize(parent) {
        Ok(resolved) => resolved,
        Err(error) => {
            tracing::debug!(parent = %parent.display(), %error, "the folder of a trashed path cannot be resolved");
            return None;
        }
    };
    return Some(resolved.join(name));
}

/// What: `same_original_path` tells whether a trash item came from `wanted`, comparing the paths as written first and
/// then with their folders resolved through symlinks.
/// Why: Undo must find the item however the library path was spelled; see `comparable_path`.
/// In TS you'd write (pseudocode):
/// ```ts
/// const sameOriginalPath = (a: string, b: string) => a === b || comparablePath(a) === comparablePath(b);
/// ```
pub fn same_original_path(original: &Path, wanted: &Path) -> bool {
    if original == wanted {
        return true;
    }
    let (Some(left), Some(right)) = (comparable_path(original), comparable_path(wanted)) else {
        return false;
    };
    return left == right;
}

/// How long a trashed path counts as "just trashed".
const JUST_TRASHED: std::time::Duration = std::time::Duration::from_secs(15);

/// Paths moved to the trash by this player and when.
static TRASHED: std::sync::Mutex<Vec<(PathBuf, std::time::Instant)>> = std::sync::Mutex::new(Vec::new());

/// What: `was_just_trashed` tells whether this player moved `path` to the trash in the last few seconds.
/// Why: the rescan then reports the file as gone, and the trash message already explains it; a second message about
/// the same file would be noise.
pub fn was_just_trashed(path: &Path) -> bool {
    let Ok(mut trashed) = TRASHED.lock() else {
        return false;
    };
    trashed.retain(|(_path, when)| return when.elapsed() < JUST_TRASHED);
    return trashed.iter().any(|(trashed_path, _when)| return trashed_path == path);
}

/// What: `can_restore_from_trash` tells whether `restore_from_trash` works on this platform.
/// Why: the Undo message is offered only where the trash crate can restore a file.
pub fn can_restore_from_trash() -> bool {
    return cfg!(any(
        windows,
        all(unix, not(any(target_os = "macos", target_os = "ios", target_os = "android")))
    ));
}

/// What: `restore_from_trash` puts the most recently trashed copy of `path` back where it was.
/// Why: Undo of Move to trash restores the file from the OS trash, so nothing is copied or re-created.
#[cfg(any(
    windows,
    all(unix, not(any(target_os = "macos", target_os = "ios", target_os = "android")))
))]
pub fn restore_from_trash(path: &Path) -> Result<(), FileActionError> {
    let name: String = path.file_name().map_or_else(|| return path.display().to_string(), |name| return name.to_string_lossy().into_owned());
    let items = trash::os_limited::list().map_err(|error| {
        tracing::warn!(path = %path.display(), %error, "the trash could not be listed for Undo");
        return FileActionError::Restore(format!("{name} could not be put back: the trash could not be read."));
    })?;
    let newest = items
        .into_iter()
        .filter(|item| return same_original_path(&item.original_path(), path))
        .max_by_key(|item| return item.time_deleted)
        .ok_or_else(|| {
            tracing::warn!(path = %path.display(), "no item in the trash came from this path");
            return FileActionError::Restore(format!("{name} is not in the trash any more, so it could not be put back."));
        })?;
    return trash::os_limited::restore_all([newest]).map_err(|error| {
        tracing::warn!(path = %path.display(), %error, "the trash could not restore the file");
        return FileActionError::Restore(format!("{name} could not be put back from the trash."));
    });
}

/// What: `restore_from_trash` on macOS reports that the trash crate cannot restore files there.
/// Why: the Undo message is not offered on this platform, so this branch is a guard only.
#[cfg(not(any(
    windows,
    all(unix, not(any(target_os = "macos", target_os = "ios", target_os = "android")))
)))]
pub fn restore_from_trash(_path: &Path) -> Result<(), FileActionError> {
    return Err(FileActionError::Restore("the trash cannot be restored on this platform".to_string()));
}

#[cfg(test)]
#[path = "ui_files_tests.rs"]
mod tests;
