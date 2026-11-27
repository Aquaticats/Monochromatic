//! What:
//!  What Git's discovery found above the walked directory,
//!  and the three answers a scan can act on:
//!  a working tree with its Git directories and the scanned directory's
//!  place below the root,
//!  no repository at all,
//!  or an entry that exists but cannot be used.
//! Why:
//!  This is the value the walker matches on,
//!  so it is also where the "cannot be used" case carries the entry and the
//!  cause for the line the scan prints about incomplete coverage.
//!  The nearest `.git` wins,
//!  which is the innermost working tree Git itself would work in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // discover(walkRoot): Discovery
//! ```

/// Imports the path type the walk starts from and the directory type it
///  answers with.
use std::path::{
    Path,
    PathBuf,
};

/// Imports what one directory in the walk upward holds as a `.git` entry.
use super::dot_git_entry::{
    RootEntry,
    git_directories,
};
/// Imports the absolute form and the scope prefix of the walked directory.
use super::path_normalization::{
    absolute_path,
    scope_bytes,
};

/// The environment variable naming the active index file.
const INDEX_FILE_ENVIRONMENT: &str = "GIT_INDEX_FILE";

/// What Git's discovery found above the walked directory.
pub(crate) enum Discovery {
    /// The walked directory is inside a working tree.
    WorkingTree(
        /// The resolved directories of that working tree.
        GitWorkingTree,
    ),
    /// No `.git` entry above the walked directory,
    ///  so nothing claims a workspace with tracked files.
    NoRepository,
    /// A `.git` entry exists above the walked directory but names nothing this
    ///  scanner can read a working tree from.
    ///
    /// Git itself stops discovery with a fatal error here;
    ///  this value exists so a scan can say the same thing while still
    ///  walking the files it can see.
    Unusable {
        /// The `.git` entry that could not be used.
        entry: PathBuf,
        /// What is wrong with it,
        /// in words.
        cause: String,
    },
}

/// The administrative directories of one working tree,
///  and the walked directory's place below its root.
pub(crate) struct GitWorkingTree {
    /// The Git directory holding this working tree's `index`.
    git_dir: PathBuf,
    /// The Git directory holding `config`:
    ///  the shared one for a linked worktree,
    ///  whose own directory holds only per-worktree state.
    common_dir: PathBuf,
    /// The walked directory's path below the working tree root,
    ///  `/`-separated with a trailing separator,
    ///  empty when the scan starts at the root.
    scope_below_root: Vec<u8>,
}

/// Finds the working tree whose root is `walk_root` or one of its ancestors.
///
/// The nearest `.git` wins,
///  which is the innermost working tree Git itself would work in.
pub(crate) fn discover(walk_root: &Path) -> Discovery {
    let Some(absolute) = absolute_path(walk_root) else {
        return Discovery::Unusable {
            entry: walk_root.to_path_buf(),
            cause: "the scan directory cannot be resolved".to_string(),
        };
    };
    for candidate in absolute.ancestors() {
        match git_directories(candidate) {
            RootEntry::None => {}
            RootEntry::Unusable {
                entry,
                cause,
            } => {
                return Discovery::Unusable {
                    entry,
                    cause,
                };
            }
            RootEntry::Tree {
                git_dir,
                common_dir,
            } => {
                let below: &Path = absolute.strip_prefix(candidate).unwrap_or(Path::new(""));
                return Discovery::WorkingTree(GitWorkingTree {
                    git_dir,
                    common_dir,
                    scope_below_root: scope_bytes(below),
                });
            }
        }
    }
    return Discovery::NoRepository;
}

/// What this working tree's index is asked for,
///  and how its entries are read.
impl GitWorkingTree {
    /// The path `gix-index` opens.
    ///
    /// `GIT_INDEX_FILE` names the active index when it is set,
    ///  relative to the current directory as Git reads it,
    ///  which is how a Git command that scans with a temporary index
    ///  (a commit in progress,
    /// for example) makes that index the one in use.
    pub(crate) fn index_path(&self) -> PathBuf {
        if let Some(named) = std::env::var_os(INDEX_FILE_ENVIRONMENT)
            && !named.is_empty()
        {
            return PathBuf::from(named);
        }
        return self.git_dir.join("index");
    }

    /// The configuration file recording this working tree's object format.
    pub(crate) fn config_path(&self) -> PathBuf {
        return self.common_dir.join("config");
    }

    /// One index entry's path expressed below the walked directory,
    ///  or `None` when the entry lies outside it.
    ///
    /// Index paths are recorded from the working tree root,
    ///  so a scan of a subfolder drops both the prefix above the walked
    ///  directory and every entry outside it;
    ///  reporting them unchanged would scan files the caller did not ask for.
    pub(crate) fn below_walk_root<'entry>(
        &self,
        entry: &'entry [u8],
    ) -> Option<&'entry [u8]> {
        if self.scope_below_root.is_empty() {
            return Some(entry);
        }
        return entry.strip_prefix(self.scope_below_root.as_slice());
    }
}
