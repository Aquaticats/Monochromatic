//! What:
//!  The folder one runtime directory keeps every worktree's commit service socket in,
//! created so that only this account may enter it,
//! and the socket file itself,
//! made readable and writable by this account alone.
//! Why:
//!   The service accepts a standards-review credential over that socket
//! (`service_review_key.rs`),
//! and a socket another account could open would hand it that credential.
//! A service manager used to make the folder with `DirectoryMode=0700` and the socket with
//! `SocketMode=0600`;
//! with the wrapper starting the server,
//! the wrapper makes them,
//! and it makes them again on every start because a folder can be there with other modes
//! (an earlier release, a person's umask, a restored runtime directory).
//! The mode is set after the folder exists rather than only at its creation,
//! so a start repairs what it finds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! mkdir(join(runtimeDir, 'monochromatic-git-policy-cli'), { recursive: true, mode: 0o700 });
//! chmodSync(directory, 0o700); chmodSync(socket, 0o600);
//! ```

use crate::service_instance::SOCKET_DIRECTORY;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// The folder the sockets live in.
pub fn path(runtime_dir: &Path) -> PathBuf {
    return runtime_dir.join(SOCKET_DIRECTORY);
}

/// What:
///  Makes the socket folder exist and enterable by this account alone.
/// Why:
///   A folder that exists with wider modes is repaired,
/// and one that cannot be made private is an error the caller reports:
///  a service that listened in a folder others can enter must not start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ensurePrivate(path(runtimeDir))
/// ```
pub fn ensure_private(folder: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(folder)?;
    let private: std::fs::Permissions = std::fs::Permissions::from_mode(0o700);
    return std::fs::set_permissions(folder, private);
}

/// What:
///  Makes a listening socket openable by this account alone.
/// Why:
///   The kernel makes a bound socket file with the process's umask applied to 0777,
/// which is 0755 on an ordinary account:
/// anyone may connect.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// chmodSync(socket, 0o600)
/// ```
pub fn keep_only_its_owner(socket: &Path) -> std::io::Result<()> {
    return std::fs::set_permissions(socket, std::fs::Permissions::from_mode(0o600));
}

/// Controls of the folder's modes stay out of the release executable.
#[cfg(test)]
#[path = "service_socket_directory_only_this_account_may_open_tests.rs"]
mod tests;
