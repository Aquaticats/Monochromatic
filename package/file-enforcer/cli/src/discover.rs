//! What:
//!  Find the repository config and the worktree root it implies.
//! Why:
//!  The worktree root is the directory holding `file-enforcer.config.jsonc`;
//! commands run from any subdirectory find
//!  the nearest one above,
//! and `--config` names one explicitly.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function findConfig(start: string): string | undefined
//! ```

/// Imports `Diagnostic` from `monochromatic_file_enforcer::diagnostic`.
use monochromatic_file_enforcer::diagnostic::Diagnostic;
/// Imports `Path`,
/// `PathBuf` from `std::path`.
use std::path::{Path, PathBuf};

/// The config file name searched for upward from the current directory.
pub const CONFIG_FILE_NAME: &str = "file-enforcer.config.jsonc";

/// What:
///  Resolve the config path and the worktree root.
///  `(PathBuf, PathBuf)` is a pair of owned paths (TS:
/// `[string, string]`).
/// Why:
///  An explicit `--config` wins;
/// otherwise the nearest config above the current directory is used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveRepository(explicit?: string): [root: string, config: string]
/// ```
pub fn resolve_repository(explicit: &Option<PathBuf>) -> Result<(PathBuf, PathBuf), Diagnostic> {
    let current = match std::env::current_dir() {
        Ok(current) => current,
        Err(error) => {
            let message = format!("cannot read the current directory: {error}");
            return Err(Diagnostic::new("usage-error", &message));
        }
    };
    let config = match explicit {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => current.join(path),
        None => match find_upward(&current) {
            Some(found) => found,
            None => {
                let message = format!("no {CONFIG_FILE_NAME} in {} or any directory above it", current.display());
                return Err(Diagnostic::new("config-missing", &message).with_hint("run from inside the repository or pass --config"));
            }
        },
    };
    let Some(parent) = config.parent() else {
        let message = format!("{} has no parent directory", config.display());
        return Err(Diagnostic::new("usage-error", &message));
    };
    let root = std::fs::canonicalize(parent).unwrap_or_else(|_| return parent.to_path_buf());
    let file_name = config.file_name().map(|name| return name.to_os_string()).unwrap_or_default();
    return Ok((root.clone(), root.join(file_name)));
}

/// What:
///  Walk up from a directory looking for the config file.
/// Why:
///  Mirrors how other repository tools find their root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findUpward(start: string): string | undefined
/// ```
fn find_upward(start: &Path) -> Option<PathBuf> {
    let mut directory = Some(start);
    while let Some(candidate) = directory {
        let config = candidate.join(CONFIG_FILE_NAME);
        if config.is_file() {
            return Some(config);
        }
        directory = candidate.parent();
    }
    return None;
}
