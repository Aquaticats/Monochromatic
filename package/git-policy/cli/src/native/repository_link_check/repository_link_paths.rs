//! What:
//!  The paths an invocation names,
//! and whether one lies under the repository link.
//! Why:
//!  The test is on the path the wrapper finally uses,
//! not on the text of an argument.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const touches = paths.some((path) => underRepository(identity, resolve(path)));
//! ```

use crate::repository_link_identity::LinkIdentity;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What:
///  The directories an invocation names.
/// Why:
///  Where Git will work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// namedPaths({ cwd, globalPrefix, environment })
/// ```
pub fn named_paths(cwd: &Path, global_prefix: &[OsString], environment: &[(OsString, OsString)]) -> Vec<PathBuf> {
    todo!("{} {} {}", cwd.display(), global_prefix.len(), environment.len())
}

/// What:
///  Whether a path lies under the repository.
/// Why:
///  Decides whether the check runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// touchesRepository(identity, path)
/// ```
pub fn touches_repository(identity: &LinkIdentity, path: &Path) -> bool {
    todo!("{identity:?} {}", path.display())
}

/// Path controls stay out of the release executable.
#[cfg(test)]
#[path = "repository_link_paths_tests.rs"]
mod tests;
