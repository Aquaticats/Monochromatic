//! What:
//!  Which commit service instance belongs to a worktree: a name made from the worktree's own Git directory, and the
//!  socket, the unit instances and the journal directory that name gives.
//! Why:
//!  One service instance runs per worktree, the way Bazel runs one server per workspace, so a wait in one worktree (a foreign
//!  holder of its index lock, a slow push) never delays another's commits. A worktree's own Git directory (the main
//!  worktree's `.git`, or `.git/worktrees/<id>` of a linked one) holds its `HEAD`, its index and its work records, and Git
//!  allows one branch to be checked out in one worktree only, so the instances never write the same branch
//!  (`doc/planning/cli-git-resident-service.md`, "One service per worktree"). The name is a hash because an escaped path
//!  does not fit a unit name for an ordinary home directory, and the hash is the same in the wrapper and in the service,
//!  which are built together. A collision of two worktrees would only make one instance serve both, never land anywhere
//!  wrong.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const socket = join(runtimeDir, 'cli-git', `${instanceOf(gitDir)}.sock`);
//! ```

use std::path::{Path, PathBuf};

/// Directory of the sockets under the user's runtime directory.
pub const SOCKET_DIRECTORY: &str = "cli-git";

/// The socket of the one service every worktree of an account shared until the release that gave each worktree its own. A
///  service still listening here is asked to retire and its units are removed.
pub const LEGACY_SOCKET_RELATIVE: &str = "cli-git/service.sock";

/// The number of hexadecimal digits of an instance name.
pub const INSTANCE_LENGTH: usize = 32;

/// The 128-bit FNV-1a offset basis.
const FNV_OFFSET: u128 = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d;

/// The 128-bit FNV-1a prime.
const FNV_PRIME: u128 = 0x0000_0000_0100_0000_0000_0000_0000_013b;

/// What:
///  The instance name of a worktree: 32 hexadecimal digits of the FNV-1a hash of its real Git directory path.
/// Why:
///   The real path makes two spellings of one directory (a link, a relative part) one instance. A directory that cannot be
///  resolved is hashed as given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// instanceOf(gitDir) // '3f9a...' (32 digits)
/// ```
pub fn instance_of(git_dir: &Path) -> String {
    use std::os::unix::ffi::OsStrExt;
    // A directory that cannot be resolved is named as given.
    let real: PathBuf = match std::fs::canonicalize(git_dir) {
        Ok(resolved) => resolved,
        Err(_unresolvable) => git_dir.to_path_buf(),
    };
    let mut hash: u128 = FNV_OFFSET;
    for byte in real.as_os_str().as_bytes() {
        hash ^= u128::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    return format!("{hash:032x}");
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
    return format!("cli-git@{instance}.socket");
}

/// The name of the service unit instance of a worktree.
pub fn service_unit_instance(instance: &str) -> String {
    return format!("cli-git@{instance}.service");
}

/// Instance controls stay out of the release executable.
#[cfg(test)]
#[path = "service_instance_tests.rs"]
mod tests;
