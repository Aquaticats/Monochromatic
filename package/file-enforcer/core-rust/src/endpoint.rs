//! What:
//!  Where a command's server lives: the repository's per-worktree one, or the per-user one.
//! Why:
//!  The repository server is keyed by the canonical worktree root and the per-user server by the canonical per-user
//!  config path, so the two never share a socket and neither depends on the current directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function repositoryEndpoint(root: string): Endpoint; function userEndpoint(configPath: string): Endpoint
//! ```

use crate::diagnostic::Diagnostic;
use crate::runtime_dir::{prepare_runtime_directory, runtime_directory, server_key, socket_path};
use directories::BaseDirs;
use std::path::{Path, PathBuf};

/// What:
///  The socket a command talks to and the root that server serves.
/// Why:
///  Both are needed to build the client options and the `serve` arguments.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Endpoint = { socket: string; root: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Endpoint {
    /// Socket path under the runtime directory.
    pub socket: PathBuf,
    /// The directory the server treats as its worktree root.
    pub root: PathBuf,
}

/// What:
///  Resolve and prepare the runtime directory for the current environment.
///  `std::env::var_os` reads an environment variable that may not be UTF-8 (TS: `process.env.X`).
/// Why:
///  `$XDG_RUNTIME_DIR` is used when usable; otherwise an owner-only directory under the cache directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveRuntimeDirectory(): string
/// ```
fn resolve_runtime_directory() -> Result<PathBuf, Diagnostic> {
    let xdg = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from);
    let Some(base) = BaseDirs::new() else {
        return Err(Diagnostic::new("runtime-dir-unusable", "the home directory cannot be determined"));
    };
    let directory = runtime_directory(xdg, base.cache_dir())?;
    prepare_runtime_directory(&directory)?;
    return Ok(directory);
}

/// What:
///  The endpoint for a repository worktree.
/// Why:
///  One server per worktree; the socket name derives from a hash of the canonical root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function repositoryEndpoint(root: string): Endpoint
/// ```
pub fn repository_endpoint(root: &Path) -> Result<Endpoint, Diagnostic> {
    let canonical = std::fs::canonicalize(root).unwrap_or_else(|_| return root.to_path_buf());
    let directory = resolve_runtime_directory()?;
    let key = server_key(&canonical.display().to_string());
    return Ok(Endpoint { socket: socket_path(&directory, &key)?, root: canonical });
}

/// What:
///  The endpoint for the per-user config.
/// Why:
///  `user` commands get their own server keyed by the canonical per-user config path, independent of any worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function userEndpoint(configPath: string): Endpoint
/// ```
pub fn user_endpoint(config_path: &Path) -> Result<Endpoint, Diagnostic> {
    let canonical = match std::fs::canonicalize(config_path) {
        Ok(canonical) => canonical,
        Err(error) => {
            let message = format!("cannot read the per-user config {}: {error}", config_path.display());
            return Err(Diagnostic::new("config-unreadable", &message).with_path(&config_path.display().to_string()));
        }
    };
    let Some(base) = BaseDirs::new() else {
        return Err(Diagnostic::new("runtime-dir-unusable", "the home directory cannot be determined"));
    };
    let directory = resolve_runtime_directory()?;
    let key = server_key(&format!("user:{}", canonical.display()));
    return Ok(Endpoint { socket: socket_path(&directory, &key)?, root: base.home_dir().to_path_buf() });
}

/// What:
///  The default location of the per-user config.
/// Why:
///  People can pass `--user-config`; otherwise this is `<config dir>/monochromatic-dev/file-enforcer/user.config.jsonc`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function defaultUserConfigPath(): string
/// ```
pub fn default_user_config_path() -> Result<PathBuf, Diagnostic> {
    let Some(base) = BaseDirs::new() else {
        return Err(Diagnostic::new("usage-error", "the home directory cannot be determined"));
    };
    return Ok(base.config_dir().join("monochromatic-dev").join("file-enforcer").join("user.config.jsonc"));
}
