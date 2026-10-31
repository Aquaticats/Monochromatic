//! What:
//!  Where the per-worktree server's socket and startup lock live,
//! and how they are named.
//! Why:
//!  The socket must sit in an owner-only directory (`$XDG_RUNTIME_DIR`,
//! or a verified owner-only fallback when it is
//!  unset or unusable),
//! its name derives from a hash of the worktree root,
//! and the whole path must fit the Unix socket
//!  limit,
//! so a long worktree path never produces an unbindable socket.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function runtimeDirectory(env: Record<string, string>): string
//! ```

/// Imports `Diagnostic` from `crate::diagnostic`.
use crate::diagnostic::Diagnostic;
/// Imports `sha256_hex` from `crate::real_fs`.
use crate::real_fs::sha256_hex;
/// Imports `fs` from `std`.
use std::fs;
/// Imports `DirBuilderExt`,
/// `MetadataExt` from `std::os::unix::fs`.
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
/// Imports `Path`,
/// `PathBuf` from `std::path`.
use std::path::{Path, PathBuf};

/// What:
///   The longest socket path accepted,
/// in bytes:
/// `100`.
///   `usize` is the unsigned integer that measures lengths (siblings:
/// `u32`,
/// `u64`).
/// Why:
///   The kernel's `sun_path` holds 108 bytes including the terminator,
///   so a longer path fails with `socket-path-too-long` instead of an obscure bind error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const MAX_SOCKET_PATH_BYTES = 100;
/// ```
pub const MAX_SOCKET_PATH_BYTES: usize = 100;

/// What:
///  Name a server by hashing what it serves (a worktree root,
/// or the per-user config path).
/// Why:
///  The name is short and fixed-length,
/// whatever the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function serverKey(subject: string): string // first 16 hex digits of sha256
/// ```
pub fn server_key(subject: &str) -> String {
    return sha256_hex(subject.as_bytes()).chars().take(16).collect();
}

/// What:
///  Check that a directory is a real directory owned by this user with no group or other access.
/// Why:
///  A socket in a shared directory could be replaced or reached by other users.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function verifyOwnerOnly(dir: string): void
/// ```
pub fn verify_owner_only(dir: &Path) -> Result<(), Diagnostic> {
    let metadata = match fs::symlink_metadata(dir) {
        Ok(metadata) => metadata,
        Err(error) => {
            let message = format!("cannot inspect the runtime directory {}: {error}", dir.display());
            return Err(Diagnostic::new("runtime-dir-unusable", &message));
        }
    };
    // SAFETY: `geteuid` takes no arguments, has no preconditions and cannot fail.
    let owner = unsafe { libc::geteuid() };
    if !metadata.is_dir() || metadata.uid() != owner || metadata.mode() & 0o077 != 0 {
        let message = format!(
            "{} is not a directory owned by the current user with mode 0700 (no group or other access)",
            dir.display()
        );
        return Err(Diagnostic::new("runtime-dir-unusable", &message));
    }
    return Ok(());
}

/// What:
///  Create a directory (and missing parents) with mode 0700.
/// Why:
///  The fallback directory must be owner-only from the moment it exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function createPrivate(dir: string): void
/// ```
fn create_private(dir: &Path) -> Result<(), Diagnostic> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true).mode(0o700);
    if let Err(error) = builder.create(dir) {
        let message = format!("cannot create the runtime directory {}: {error}", dir.display());
        return Err(Diagnostic::new("runtime-dir-unusable", &message));
    }
    return Ok(());
}

/// What:
///  Choose the runtime directory:
/// `xdg` when it verifies,
/// else the owner-only fallback under the cache directory.
///  `Option<PathBuf>` is "a path or nothing" (TS:
/// `string | undefined`).
/// Why:
///  `$XDG_RUNTIME_DIR` is unset in minimal containers and sometimes points at a directory the user cannot use;
/// the
///  server must still start there,
/// deterministically.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runtimeDirectory(xdg: string | undefined, fallback: string): string
/// ```
pub fn runtime_directory(xdg: Option<PathBuf>, fallback_base: &Path) -> Result<PathBuf, Diagnostic> {
    if let Some(candidate) = xdg
        && candidate.is_absolute()
        && verify_owner_only(&candidate).is_ok()
    {
        return Ok(candidate.join("file-enforcer"));
    }
    let fallback = fallback_base.join("file-enforcer").join("run");
    create_private(&fallback)?;
    verify_owner_only(&fallback)?;
    return Ok(fallback);
}

/// What:
///  Create (if needed) and verify the directory that holds this program's sockets.
/// Why:
///  `runtime_directory` names the directory;
/// this makes it exist with mode 0700.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prepareRuntimeDirectory(dir: string): void
/// ```
pub fn prepare_runtime_directory(dir: &Path) -> Result<(), Diagnostic> {
    if !dir.exists() {
        create_private(dir)?;
    }
    return verify_owner_only(dir);
}

/// What:
///  Build the socket path for a key and reject one that is too long for the kernel.
/// Why:
///  The kernel fails `bind` with a cryptic error when the path exceeds `sun_path`;
/// a named diagnostic is better.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function socketPath(dir: string, key: string): string
/// ```
pub fn socket_path(dir: &Path, key: &str) -> Result<PathBuf, Diagnostic> {
    let path = dir.join(format!("{key}.sock"));
    let length = path.as_os_str().len();
    if length > MAX_SOCKET_PATH_BYTES {
        let message = format!(
            "the socket path {} is {length} bytes; Unix sockets allow at most {MAX_SOCKET_PATH_BYTES}",
            path.display()
        );
        return Err(Diagnostic::new("socket-path-too-long", &message).with_hint("set XDG_RUNTIME_DIR to a shorter owner-only directory"));
    }
    return Ok(path);
}

/// What:
///  The startup-lock path beside a socket path.
/// Why:
///  The lock is a separate file so its holder can remove a stale socket without racing the server's own bind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lockPath(socket: string): string
/// ```
pub fn lock_path(socket: &Path) -> PathBuf {
    return socket.with_extension("lock");
}
