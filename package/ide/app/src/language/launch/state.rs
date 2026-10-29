//! The private state directory rules of the launch seam:
//!  where a state directory may lie,
//!  and the directories a launch creates and empties before it starts.

/// A launch lists the directories to create and the scratch directories to empty.
use super::{
    ServerLaunch,
    scratch::{claim, reap_ended},
};
/// `Path` is a borrowed filesystem path and `PathBuf` its owned sibling.
use std::path::{Path, PathBuf};

/// What:
///  Resolve as much of a path as exists.
///  `canonicalize` fails for a missing path,
///  so the
///       nearest existing ancestor is resolved and the missing tail is appended again.
/// Why:
///  A state directory that does not exist yet must still be compared,
///  and bound inside a
///      sandbox,
///  by its real location,
///  even when a variable spells it through a symbolic link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveExisting(path: string): string
/// ```
pub fn resolve_existing(path: &Path) -> PathBuf {
    // `mut` allows walking up until an existing ancestor is found.
    let mut existing = path;
    let mut tail: Vec<&std::ffi::OsStr> = Vec::new();
    loop {
        // `if let Ok(x) = ...` runs the block only when the call succeeded.
        if let Ok(real) = std::fs::canonicalize(existing) {
            let mut resolved = real;
            // `iter().rev()` walks the collected names from the outermost to the innermost.
            for name in tail.iter().rev() {
                resolved.push(name);
            }
            return resolved;
        }
        match (existing.parent(), existing.file_name()) {
            (Some(parent), Some(name)) => {
                tail.push(name);
                existing = parent;
            }
            // Nothing above exists or the path has no name: keep the path as written.
            _ => return path.to_path_buf(),
        }
    }
}

/// What:
///  Check that a directory a policy wants written does not contain the project.
///       `Result<(), String>` is success without a value,
///  or the reason for refusal.
/// Why:
///  A state directory that contains (or is) the project would make the project writable
///      through it (measured in `doc/planning/slint-ide-write-confinement.md`).
///  A state directory
///      strictly inside the project is allowed:
///  it is the application's own cache,
///  which lies inside
///      the project when the home folder is opened (`~/.cache/monochromatic-ide`),
///  and the sandbox
///      binds it writable after the read-only project bind,
///  so only that directory is writable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkStateDirectory(directory: string, projectRoot: string): void // throws to refuse
/// ```
pub fn check_state_directory(directory: &Path, project_root: &Path) -> Result<(), String> {
    if !directory.is_absolute() {
        // `Err(...)` is the failure variant of `Result`.
        return Err(format!(
            "private state directory {} is not an absolute path",
            directory.display()
        ));
    }
    let state = resolve_existing(directory);
    let project = resolve_existing(project_root);
    if state != project && state.starts_with(&project) {
        tracing::debug!(state = %state.display(), project = %project.display(), "private state directory lies inside the project; it is bound writable after the read-only project");
    }
    // Equal paths also start with each other, so the project itself is refused here too.
    if project.starts_with(&state) {
        return Err(format!(
            "private state directory {} contains the project {}",
            state.display(),
            project.display()
        ));
    }
    // `Ok(())` reports success without a value.
    return Ok(());
}

/// What:
///  Check that a directory lies strictly below the private state root.
/// Why:
///  Scratch directories are emptied,
///  so a policy mistake must never reach beyond the one
///      directory the application owns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkBelowState(directory: string, stateRoot: string): void // throws to refuse
/// ```
fn check_below_state(directory: &Path, state_root: &Path) -> Result<(), String> {
    let resolved = resolve_existing(directory);
    let state = resolve_existing(state_root);
    if resolved == state || !resolved.starts_with(&state) {
        return Err(format!(
            "{} is not below the private state directory {}",
            resolved.display(),
            state.display()
        ));
    }
    return Ok(());
}

/// What:
///  Create a launch's directories and empty its scratch directories.
///  `Option<&Path>` is
///       "a borrowed path,
///  or nothing":
///  the application's private state root.
/// Why:
///  A wrapper refuses to start when its bind sources are missing,
///  and a private temporary
///      directory must not carry files from an earlier session.
///  Every directory must be below
///      the state root,
///  and the state root must not contain the project;
///  anything else is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prepare(launch: ServerLaunch, projectRoot: string, stateRoot?: string): void // throws to refuse
/// ```
pub fn prepare(
    launch: &ServerLaunch,
    project_root: &Path,
    state_root: Option<&Path>,
) -> Result<(), String> {
    if launch.directories.is_empty() && launch.scratch.is_empty() {
        return Ok(());
    }
    // `let Some(x) = option else { ... }` binds the inner value or leaves the function.
    let Some(state) = state_root else {
        return Err(
            "the launch needs private directories, but no private state directory is configured"
                .to_string(),
        );
    };
    check_state_directory(state, project_root)?;
    for directory in &launch.directories {
        check_below_state(directory, state)?;
        // What: `map_err` converts the I/O error into the text reason this function returns;
        //       the trailing `?` returns that reason to the caller on failure.
        // Why: The reason is shown as the server's refused state.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // mkdirSync(directory, { recursive: true });
        // ```
        if let Err(error) = std::fs::create_dir_all(directory) {
            return Err(format!(
                "cannot create {}: {error}. The server is not started without its private state directory. Make that path creatable as a directory (free space, write permission, no file in its place), then restart the application",
                directory.display()
            ));
        }
    }
    for directory in &launch.scratch {
        check_below_state(directory, state)?;
        claim(directory)?;
        empty_in_place(directory)?;
        if let Err(error) = std::fs::create_dir_all(directory) {
            return Err(format!("cannot create {}: {error}", directory.display()));
        }
        reap_ended(directory);
    }
    return Ok(());
}

/// The reason text of a scratch directory that could not be emptied.
fn emptying_refused(path: &Path, error: &std::io::Error) -> String {
    return format!("cannot empty {}: {error}", path.display());
}

/// What:
///  Remove everything inside a scratch directory and keep the directory itself.
///  A path that is a symbolic link or a file is replaced by a real directory
///  and never followed.
/// Why:
///  The directory is the source of a `--bind` of the sandbox.
///  Another process on the same project
///  (a second window,
///  or a server and a mise run starting together)
///  prepares the same path while this process is about to start bubblewrap,
///  and a directory removed and made again is missing in between:
///  bubblewrap then stops with `Can't bind mount ... No such file or directory`.
///  A directory emptied in place is never missing,
///  and the sandbox that already holds it keeps a directory that exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function emptyInPlace(directory: string): void // throws a reason
/// ```
fn empty_in_place(directory: &Path) -> Result<(), String> {
    let Ok(found) = std::fs::symlink_metadata(directory) else {
        // Nothing there yet (or nothing readable): the caller creates it.
        return Ok(());
    };
    if !found.is_dir() {
        // A link or a file where the directory belongs: remove the name only, never what a link points at.
        if let Err(error) = std::fs::remove_file(directory) {
            return Err(emptying_refused(directory, &error));
        }
        return Ok(());
    }
    let entries = match std::fs::read_dir(directory) {
        Ok(entries) => entries,
        // Removed by another process since the check: the caller creates it.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(emptying_refused(directory, &error)),
    };
    for listed in entries {
        let entry = match listed {
            Ok(entry) => entry,
            Err(error) => return Err(emptying_refused(directory, &error)),
        };
        let path = entry.path();
        let is_directory = match entry.file_type() {
            Ok(kind) => kind.is_dir(),
            Err(error) => return Err(emptying_refused(&path, &error)),
        };
        let removed = if is_directory {
            std::fs::remove_dir_all(&path)
        } else {
            std::fs::remove_file(&path)
        };
        match removed {
            Ok(()) => {}
            // Another process emptied it at the same time.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(emptying_refused(&path, &error)),
        }
    }
    return Ok(());
}
