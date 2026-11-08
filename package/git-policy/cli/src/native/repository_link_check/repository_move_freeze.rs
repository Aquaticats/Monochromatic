//! What:
//!  Whether a repository move has frozen a repository,
//! read from the move's own record:
//! the freeze manifest the runner writes into its state folder when it sets the subvolumes read-only,
//! and renames aside when it thaws them.
//! Why:
//!  A commit on a read-only filesystem fails with the operating system's `EROFS`,
//! which has many causes;
//! the person is told a move is running only when the move's record says it froze this repository
//! (`package/repository-move/cli/src/freeze-pass.ts`,
//! `FREEZE_MANIFEST`),
//! never from the identity file alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const frozen = manifestRows(await readFile(join(stateFolder, 'freeze-manifest.tsv'))).some(({ path }) => isUnder(repository, path));
//! ```

use std::path::{Path, PathBuf};

/// The file name of the freeze manifest in the runner's state folder (`FREEZE_MANIFEST` in the runner's `constants.ts`).
pub const FREEZE_MANIFEST_NAME: &str = "freeze-manifest.tsv";

/// The runner's state folder below the home directory when `MOVE_STATE_DIR` is not set
/// (`guided-config.ts`, `moveState`).
pub const DEFAULT_STATE_FOLDER_NAME: &str = "repository-move-state";

/// What:
///  The runner's state folder for this process:
/// `MOVE_STATE_DIR` when set,
/// else `repository-move-state` below the home directory.
/// Why:
///  The runner and the wrapper read the same two variables.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.env.MOVE_STATE_DIR ?? join(homedir(), 'repository-move-state')
/// ```
pub fn move_state_folder() -> Option<PathBuf> {
    todo!()
}

/// What:
///  Whether the freeze manifest in `state_folder` lists a subvolume that holds `repository`.
/// Why:
///  Decides whether a read-only failure is the move's freeze.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// freezeCovers(stateFolder, repository)
/// ```
pub fn freeze_covers(state_folder: &Path, repository: &Path) -> bool {
    todo!("{} {}", state_folder.display(), repository.display())
}

/// Freeze record controls stay out of the release executable.
#[cfg(test)]
#[path = "repository_move_freeze_tests.rs"]
mod tests;
