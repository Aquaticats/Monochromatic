//! What:
//!  The permission bits that keep a file or a folder to the account that made it,
//!  the folders the wrapper creates with them,
//!  and the removal of a folder tree it retires.
//! Why:
//!  Lock records,
//!  transaction journals and stores are private to the account,
//!  and the mode a file is created with is filtered by the process `umask`,
//!  so an explicit change makes the mode exactly private whatever the caller's `umask` is,
//!  as the incumbent's `protectPath` does.
//!  Windows keeps its inherited access rules (recorded under
//!  "Choices open to veto" in `doc/handover/cli-git-native-transactions.md`).
//!  A retired lock or transaction is a tree removed by name,
//!  and a tree another process removed first is the same end state.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await chmod(path, 0o600); await mkdir(dir, { recursive: true, mode: 0o700 }); await rm(tree, { force: true, recursive: true });
//! ```

/// A borrowed filesystem path of raw bytes.
use std::path::Path;

/// Mode of every private directory the wrapper creates:
///  owner only.
pub const PRIVATE_DIRECTORY_MODE: u32 = 0o700;

/// Mode of every private file the wrapper creates:
///  owner read and write only.
pub const PRIVATE_FILE_MODE: u32 = 0o600;

/// What:
///  Set a path's permission bits to `mode`,
///  on Unix only.
///       `std::io::Result<()>` is "nothing,
///  or an operating-system error".
/// Why:
///   The creation mode is filtered by the process `umask`;
///  an explicit change makes the
///       mode exactly private whatever the caller's `umask` is,
///  as the incumbent's
///       `protectPath` does.
///  Windows keeps its inherited access rules (recorded under
///       "Choices open to veto" in `doc/handover/cli-git-native-transactions.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (process.platform !== 'win32') await chmod(path, mode);
/// ```
pub fn protect_path(
    path: &Path,
    mode: u32,
) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        // The trait adds `.from_mode` to permissions on Unix.
        /// The Unix permission extensions,
        ///  for the mode.
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))?;
    }
    // `let _ = mode;` marks the argument as used where no mode exists.
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    let _ = (path, mode);
    return Ok(());
}

/// What:
///  Sets the permission bits of the staging file when a mode was asked for,
/// and does nothing otherwise.
/// Why:
///  `write_then_publish` applies the mode before the file appears at its final name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (mode !== undefined) { await chmod(staged, mode); }
/// ```
pub(super) fn protect_when_asked(
    staged: &Path,
    mode: Option<u32>,
) -> std::io::Result<()> {
    return match mode {
        Some(bits) => protect_path(staged, bits),
        None => Ok(()),
    };
}

/// What:
///  Create one directory with mode `0700`;
///  its parent must exist.
/// Why:
///   Lock candidates,
///  transaction directories and stores are private to the account.
///       Only the creation mode is applied,
///  as the incumbent's `mkdir(path, { mode })`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(path, { mode: 0o700 });
/// ```
pub fn create_private_directory(path: &Path) -> std::io::Result<()> {
    return private_directory_builder().create(path);
}

/// What:
///  Create a directory and every missing parent with mode `0700`;
///  an existing directory is accepted.
/// Why:
///   The per-user state directory may not exist yet,
///  and the XDG rules ask for `0700` when a program creates one.
///  Only the directories this call creates get the mode;
///  existing parents keep theirs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(path, { recursive: true, mode: 0o700 });
/// ```
pub fn create_private_directory_all(path: &Path) -> std::io::Result<()> {
    // `mut` allows the builder to be switched to recursive creation.
    let mut builder: std::fs::DirBuilder = private_directory_builder();
    builder.recursive(true);
    return builder.create(path);
}

/// What:
///  The directory builder that creates private directories on this platform.
/// Why:
///   Only Unix configures a mode;
///  a separate function per platform keeps each body free of
///       a `mut` the other platform would leave unused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const builder = { mode: 0o700 };
/// ```
#[cfg(unix)]
fn private_directory_builder() -> std::fs::DirBuilder {
    // The trait adds `.mode` to directory builders on Unix.
    /// The Unix directory-builder extensions,
    ///  for the mode.
    use std::os::unix::fs::DirBuilderExt;
    // `mut` allows the builder to be configured before it is returned.
    let mut builder: std::fs::DirBuilder = std::fs::DirBuilder::new();
    builder.mode(PRIVATE_DIRECTORY_MODE);
    return builder;
}

/// What:
///  The directory builder that creates private directories on this platform.
/// Why:
///   Windows has no mode to configure,
///  as `private_directory_builder` on Unix explains.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const builder = {};
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
fn private_directory_builder() -> std::fs::DirBuilder {
    return std::fs::DirBuilder::new();
}

/// What:
///  Remove a directory tree,
///  treating an already absent tree as removed.
/// Why:
///   Retired locks and transactions are removed by name after a rename;
///  another process
///       may have removed them first,
///  which is the same end state (`rm -rf` with `force`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(path, { recursive: true, force: true });
/// ```
pub fn remove_tree(path: &Path) -> std::io::Result<()> {
    match std::fs::remove_dir_all(path) {
        Ok(()) => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    }
}
