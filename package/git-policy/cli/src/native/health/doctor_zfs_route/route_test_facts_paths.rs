//! What:
//!  One path the route installs or reads,
//!  at the state a healthy route shows,
//!  and the two helpers that give a root-owned file and a root-owned folder.
//! Why:
//!  The handler's and the mount's checks judge a path's owner and mode,
//!  so every fact about one is built here,
//!  and a control changes the owner or the mode of the one path it judges.
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const rootFile = (path: string) => ({ path, state: { kind: 'file', owner: 0, mode: 0o755 } });
//! const rootDirectory = (path: string, mode: number) => ({ path, state: { kind: 'directory', owner: 0, mode } });
//! ```

use super::handler_checks::{
    PathFacts,
    PathState,
    ROOT_OWNER,
};
use std::path::PathBuf;

/// One path the route installs or reads,
/// at the state a healthy route shows.
pub(crate) fn path_facts(
    path: &str,
    state: PathState,
) -> PathFacts {
    return PathFacts {
        path: PathBuf::from(path),
        state,
    };
}

/// A root-owned executable file.
pub(crate) fn root_file(path: &str) -> PathFacts {
    return path_facts(
        path,
        PathState::File {
            owner: ROOT_OWNER,
            mode: 0o755,
        },
    );
}

/// A root-owned directory at the given mode.
pub(crate) fn root_directory(
    path: &str,
    mode: u32,
) -> PathFacts {
    return path_facts(
        path,
        PathState::Directory {
            owner: ROOT_OWNER,
            mode,
        },
    );
}
