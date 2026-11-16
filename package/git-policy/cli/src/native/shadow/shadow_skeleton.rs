//! What:
//!  The files and directories of an empty bare repository on the files ref backend,
//! created in-process.
//! Why:
//!  The shadow repository used to be made by `git init --bare --template=`,
//! a Git start whose whole effect is these
//!  few entries;
//! `create_shadow_repository` overwrites the `config` it writes and every later shadow step sets the
//!  `HEAD` it writes,
//! so nothing else of Git's work is used.
//!  Creating them here removes one process start from every commit (`init`;
//! the `config` read that follows it is the
//!  wrapper's own and stays).
//!  A torn creation leaves a directory that is not a repository;
//! recovery removes the shadow tree as a whole
//!  (`binary_recovery_tearing_shadow_tests.rs`),
//! exactly as it does for a torn `git init`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const directory of SKELETON_DIRECTORIES) await mkdir(join(shadow, directory), { recursive: true });
//! await writeFile(join(shadow, 'HEAD'), PLACEHOLDER_HEAD);
//! ```

/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// Directories a bare repository has,
/// relative to the repository,
/// parents before children.
///  The empty name is the repository directory itself.
///  `objects/pack`, `refs/heads` and `refs/tags` are left to Git:
///  a files-backend shadow writes no pack and needs no tag store,
///  Git makes a loose object's own directory under `objects` and the loose ref it writes under `refs`,
///  and the claim answers "no pack and no ref" by one question about each absent directory
///  (`shadow_pool_member.rs`),
///  so three directories a creation made for nothing are gone from it and from every member of the pool.
///  Measured, in a repository on this volume:
///  a claim asked 47 questions against the creation's tree and asks 23
///  (`doc/planning/git-policy-cli-shadow-pool-claim-rename-options.md`,
///  "E. The claim keeps every check, with a smaller skeleton and one listing of the common directory"),
///  and a probe of Git and the wrapper's own ref store in a shadow without the three found both working:
///  `git commit --allow-empty` landed and `for-each-ref`, `log`, `rev-parse HEAD` and `status` answered.
pub const SKELETON_DIRECTORIES: [&str; 4] = ["", "objects", "objects/info", "refs"];

/// The `HEAD` a new repository starts with:
///  a branch that does not exist yet.
///  The shadow's ref steps point `HEAD` at the transaction's own target before any Git command reads it.
pub const PLACEHOLDER_HEAD: &str = "ref: refs/heads/main\n";

/// What:
///  Creates the directories and the `HEAD` of an empty bare repository on the files ref backend.
/// Why:
///   A directory another run of the same transaction already made is not an error,
/// as with `git init`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await createShadowSkeleton(shadowPath);
/// ```
pub fn create_shadow_skeleton(shadow: &Path) -> std::io::Result<()> {
    for relative in SKELETON_DIRECTORIES {
        match std::fs::create_dir(shadow.join(relative)) {
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            other => other?,
        }
    }
    return std::fs::write(shadow.join("HEAD"), PLACEHOLDER_HEAD);
}

/// Skeleton controls stay out of the release executable.
#[cfg(test)]
#[path = "shadow_skeleton_tests.rs"]
mod tests;
