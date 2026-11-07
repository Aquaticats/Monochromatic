//! What:
//!  Which commit service instance belongs to a worktree:
//! a name made from the worktree's own Git directory,
//! and the
//!  socket,
//! the unit instances and the journal directory that name gives.
//! Why:
//!  One service instance runs per worktree,
//! the way Bazel runs one server per workspace,
//! so a wait in one worktree (a foreign
//!  holder of its index lock,
//! a slow push) never delays another's commits.
//! A worktree's own Git directory (the main
//!  worktree's `.git`,
//! or `.git/worktrees/<id>` of a linked one) holds its `HEAD`,
//! its index and its work records,
//! and Git
//!  allows one branch to be checked out in one worktree only,
//! so the instances never write the same branch
//!  (`doc/planning/cli-git-resident-service.md`,
//! "One service per worktree").
//! The name is a hash because an escaped path
//!  does not fit a unit name for an ordinary home directory,
//! and the hash is the same in the wrapper and in the service,
//!  which are built together.
//! A collision of two worktrees would only make one instance serve both,
//! never land anywhere
//!  wrong.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const socket = join(runtimeDir, 'monochromatic-git-policy-cli', `${instanceOf(gitDir)}.sock`);
//! ```

use crate::wrapper_names_old_and_new::SERVICE_UNIT;
use std::path::{Path, PathBuf};

/// Directory of the sockets under the user's runtime directory.
pub const SOCKET_DIRECTORY: &str = SERVICE_UNIT.current;

/// The socket of the one service every worktree of an account shared until the release that gave each worktree its own.
/// A
///  service still listening here is asked to retire and its units are removed.
/// It is a file of the earlier program,
/// in the
///  earlier program's folder (`SERVICE_UNIT.earlier`,
/// tag `monochromatic-git-policy-cli-v0.5.34`,
/// `service_instance.rs` line
///  26),
/// and keeps that spelling;
/// a rename of this release's names does not reach it.
pub const LEGACY_SOCKET_RELATIVE: &str = "cli-git/service.sock";

/// The number of hexadecimal digits of an instance name.
pub const INSTANCE_LENGTH: usize = 32;

/// The 128-bit FNV-1a offset basis.
const FNV_OFFSET: u128 = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d;

/// The 128-bit FNV-1a prime.
const FNV_PRIME: u128 = 0x0000_0000_0100_0000_0000_0000_0000_013b;

/// What:
///  The instance name of a worktree:
/// 32 hexadecimal digits of the FNV-1a hash of its real Git directory path.
/// Why:
///   The real path makes two spellings of one directory (a link,
/// a relative part) one instance.
/// A directory that cannot be
///  resolved is hashed as given:
/// never use this to name anything new (a socket,
/// a unit instance,
/// a journal directory);
/// `try_instance_of` refuses such a directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// instanceOf(gitDir) // '3f9a...' (32 digits)
/// ```
pub fn instance_of(git_dir: &Path) -> String {
    // A directory that cannot be resolved is named as given.
    let real: PathBuf = match std::fs::canonicalize(git_dir) {
        Ok(resolved) => resolved,
        Err(_unresolvable) => git_dir.to_path_buf(),
    };
    return instance_named_by(real.as_path());
}

/// The 32 hexadecimal digits of the FNV-1a hash of a path's bytes.
fn instance_named_by(real: &Path) -> String {
    use std::os::unix::ffi::OsStrExt;
    let mut hash: u128 = FNV_OFFSET;
    for byte in real.as_os_str().as_bytes() {
        hash ^= u128::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    return format!("{hash:032x}");
}

/// What:
///  Why a Git directory has no instance name:
/// it cannot be resolved to a real path.
/// Why:
///  A link to a drive that is not mounted names nothing,
/// and a name made from its text would make a socket,
/// a unit instance and a journal directory for a place that is not there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class GitDirUnreachable extends Error { path: string; }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitDirUnreachable {
    /// The Git directory as given.
    pub path: PathBuf,
    /// The operating system's reason,
    /// such as `No such file or directory (os error 2)`.
    pub reason: String,
}

impl std::fmt::Display for GitDirUnreachable {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(formatter, "the Git directory {} cannot be resolved: {}", self.path.display(), self.reason);
    }
}

impl std::error::Error for GitDirUnreachable {}

/// What:
///  The instance name of a worktree,
/// or why the Git directory has none.
/// Why:
///  Name a socket,
/// a unit instance or a journal directory only through this function:
/// it refuses a directory that cannot be resolved,
/// where `instance_of` would hash the text as given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// tryInstanceOf(gitDir) // '3f9a...' or throws GitDirUnreachable
/// ```
pub fn try_instance_of(git_dir: &Path) -> Result<String, GitDirUnreachable> {
    return match std::fs::canonicalize(git_dir) {
        Ok(real) => Ok(instance_named_by(real.as_path())),
        Err(error) => Err(GitDirUnreachable { path: git_dir.to_path_buf(), reason: error.to_string() }),
    };
}

/// The socket of an instance under the user's runtime directory.
pub fn socket_path(runtime_dir: &Path, instance: &str) -> PathBuf {
    return runtime_dir.join(SOCKET_DIRECTORY).join(format!("{instance}.sock"));
}

/// The socket of the worktree whose own Git directory is `git_dir`.
pub fn socket_for(runtime_dir: &Path, git_dir: &Path) -> PathBuf {
    return socket_path(runtime_dir, instance_of(git_dir).as_str());
}

/// The name of the socket unit instance of a worktree.
pub fn socket_unit_instance(instance: &str) -> String {
    return format!("{}@{instance}.socket", SERVICE_UNIT.current);
}

/// The name of the service unit instance of a worktree.
pub fn service_unit_instance(instance: &str) -> String {
    return format!("{}@{instance}.service", SERVICE_UNIT.current);
}

/// The name of the socket unit instance that the earlier release (`cli-git@<instance>.socket`) gave the same worktree.
/// The
///  instance is the same text as this release's:
/// both hash the real path of the worktree's own Git directory with the same
///  FNV-1a constants (tag `monochromatic-git-policy-cli-v0.5.34`,
/// `service_instance.rs`,
/// `instance_of` and `socket_unit_instance`),
///  so the earlier unit of one worktree is never the unit of another.
pub fn earlier_socket_unit_instance(instance: &str) -> String {
    return format!("{}@{instance}.socket", SERVICE_UNIT.earlier);
}

/// The name of the service unit instance that the earlier release gave the same worktree.
pub fn earlier_service_unit_instance(instance: &str) -> String {
    return format!("{}@{instance}.service", SERVICE_UNIT.earlier);
}

/// The socket file of the earlier release's unit for an instance (`ListenStream=%t/cli-git/%i.sock`),
/// which the manager makes
///  and removes.
pub fn earlier_socket_path(runtime_dir: &Path, instance: &str) -> PathBuf {
    return runtime_dir.join(SERVICE_UNIT.earlier).join(format!("{instance}.sock"));
}

/// Instance controls stay out of the release executable.
#[cfg(test)]
#[path = "service_instance_tests.rs"]
mod tests;
