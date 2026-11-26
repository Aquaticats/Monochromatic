//! What:
//!  The shadow's own directories and files,
//!  made private to the account:
//!  the parent directories a shadow is created in,
//!  and the mode of the shadow directory and of each file overwritten in it.
//! Why:
//!  A shadow holds the transaction's objects and refs,
//!  and a pool member is a shadow whose owner is not known until a commit claims it,
//!  so every directory and file of one is private.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await mkdir(dirname(shadow), { recursive: true, mode: 0o700 });
//! await writeFile(path, bytes, { mode: 0o600 });
//! ```

/// Create a directory no other account can enter.
use crate::private_storage::create_private_directory;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  Overwrites a file the shadow's `git init` already created,
/// with private mode.
/// Why:
///   The incumbent's `writeFile` replaces the template `config` and `alternates`,
///  while the exclusive writer used for new files would refuse them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeFile(path, bytes, { mode: 0o600 });
/// ```
pub(crate) fn overwrite_private(
    path: &Path,
    bytes: &[u8],
) -> std::io::Result<()> {
    std::fs::write(path, bytes)?;
    return set_private_file(path);
}

/// What:
///  Gives a file the private file mode on Unix.
/// Why:
///   The shadow config names the worktree root,
/// includes the real common config and switches
///  every hook off;
/// it stays private to the account.
///  It names no hooks directory:
/// every shadow command passes its own `core.hooksPath`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await chmod(path, 0o600);
/// ```
#[cfg(unix)]
fn set_private_file(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    return std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
}

/// Windows has no mode bits for this file;
/// the incumbent's ACL work is not ported.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(windows)]
fn set_private_file(_path: &Path) -> std::io::Result<()> {
    return Ok(());
}

/// What:
///  Creates the shadow's parent directories as private directories.
/// Why:
///   The incumbent creates `monochromatic-git-policy-cli/shadow` with mode 0700.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(dirname(shadowPath), { recursive: true, mode: 0o700 });
/// ```
pub(crate) fn create_parent(shadow: &Path) -> std::io::Result<()> {
    let parent: &Path = shadow.parent().unwrap_or(shadow);
    if let Some(grandparent) = parent.parent() {
        ensure_private_directory(grandparent)?;
    }
    return ensure_private_directory(parent);
}

/// What:
///  Create a private directory unless another process made it first.
/// Why:
///   Concurrent commits of one repository race to create the shared `monochromatic-git-policy-cli/shadow` parents;
/// whoever loses has
///   nothing left to do.
/// A path that exists as anything but a directory is still a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(directory, { mode: 0o700 }).catch((error) => { if (error.code !== 'EEXIST') throw error; });
/// ```
pub fn ensure_private_directory(directory: &Path) -> std::io::Result<()> {
    match create_private_directory(directory) {
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists && directory.is_dir() => {
            return Ok(());
        }
        other => return other,
    }
}

/// What:
///  Gives a shadow directory the private mode.
/// Why:
///   A shadow holds the transaction's objects and refs,
/// and a pool member is a shadow whose owner is not known until a commit claims it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await chmod(shadowPath, 0o700);
/// ```
#[cfg(unix)]
pub(crate) fn set_private_directory_mode(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    return std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700));
}

/// Windows has no mode bits for the shadow directory.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(windows)]
pub(crate) fn set_private_directory_mode(_path: &Path) -> std::io::Result<()> {
    return Ok(());
}
