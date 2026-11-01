//! What:
//!  Per-run sharing of LFS repository facts across files and worker threads.
//! Why:
//!  Every Markdown file under one `.lfsconfig` uses the same endpoint,
//!  tracked patterns and
//! resolved image targets;
//!  discovering them once per repository keeps a large run from re-reading
//! the same configuration and re-hashing the same images.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // repos.contextFor(filePath, document, setting) -> LfsImageContext | undefined
//! ```

/// Import repository discovery,
///  exclusion and per-file preparation.
use crate::{
    markdown_lfs_config::LfsConfigError,
    markdown_lfs_context::{
        LfsImageContext, LfsImageRepo, discover_lfs_image_repo, find_lfs_repo_root, is_excluded,
        prepare_lfs_image_context,
    },
    markdown_rule_settings::LfsSetting,
    markdown_source::MarkdownSource,
};
/// Import an ordered map,
///  shared ownership (`Arc`,
///  usable across threads unlike `Rc`) and a lock.
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};

/// What:
///  A discovery outcome kept for the rest of the run,
///  including a failed one.
/// Why:
///  A repository whose endpoint cannot be used fails every file the same way without being re-read.
/// `None` means the root's `.lfsconfig` declares no endpoint.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Discovered = LfsImageRepo | undefined | LfsConfigError;
/// ```
type Discovered = Result<Option<Arc<LfsImageRepo>>, LfsConfigError>;

/// What:
///  Discovered repositories by root directory.
/// Why:
///  `Mutex` gives worker threads one shared map;
///  the lock is held only while a root is looked
/// up or discovered,
///  never while images are read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class LfsRepos { private byRoot = new Map<Path, Discovered>(); }
/// ```
#[derive(Debug, Default)]
pub struct LfsRepos {
    /// One entry per `.lfsconfig` directory encountered in this run.
    by_root: Mutex<BTreeMap<PathBuf, Discovered>>,
}

/// What:
///  Look up or discover the repository a Markdown file belongs to and prepare its rule context.
/// Why:
///  The rule itself performs no I/O;
///  this is the only path from a linted file to LFS facts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// repos.contextFor(filePath, document, setting)
/// ```
impl LfsRepos {
    /// Start with no discovered repositories.
    pub fn new() -> LfsRepos {
        return LfsRepos::default();
    }

    /// What:
    ///  The shared repository for a root,
    ///  discovering it on first use.
    /// Why:
    ///  Discovery reads `.lfsconfig` and `.gitattributes`;
    ///  doing it under the lock means two
    /// workers never discover the same root twice.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private repositoryAt(root: Path): Discovered
    /// ```
    fn repository_at(&self, root: &Path) -> Discovered {
        let mut by_root: MutexGuard<'_, BTreeMap<PathBuf, Discovered>> = self
            .by_root
            .lock()
            .expect("repository map lock is not poisoned");
        if let Some(known) = by_root.get(root) {
            return known.clone();
        }
        let discovered: Discovered = match discover_lfs_image_repo(root) {
            Ok(Some(repo)) => Ok(Some(Arc::new(repo))),
            Ok(None) => Ok(None),
            Err(error) => Err(error),
        };
        by_root.insert(root.to_path_buf(), discovered.clone());
        return discovered;
    }

    /// What:
    ///  The rule context for one file,
    ///  or `None` when the rule is inert for it.
    /// Why:
    ///  The rule is inert without a `.lfsconfig` ancestor,
    ///  without a declared endpoint,
    ///  or when
    /// the file matches the rule's `exclude` patterns.
    ///  Exclusion is decided before the endpoint is
    /// read.
    ///  `file_path` is absolute and lexically normal.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// contextFor(filePath: Path, document: MarkdownSource, setting: LfsSetting): LfsImageContext | undefined
    /// ```
    pub fn context_for(
        &self,
        file_path: &Path,
        document: &MarkdownSource,
        setting: &LfsSetting,
    ) -> Result<Option<LfsImageContext>, LfsConfigError> {
        let Some(directory): Option<&Path> = file_path.parent() else {
            return Ok(None);
        };
        let Some(root): Option<PathBuf> = find_lfs_repo_root(directory)? else {
            return Ok(None);
        };
        if is_excluded(&root, file_path, &setting.exclude) {
            return Ok(None);
        }
        let Some(repo): Option<Arc<LfsImageRepo>> = self.repository_at(&root)? else {
            return Ok(None);
        };
        let context: LfsImageContext = prepare_lfs_image_context(&repo, file_path, document)?;
        return Ok(Some(context));
    }
}

/// Sharing and exclusion-order controls stay outside release artifacts.
#[cfg(test)]
#[path = "run_lfs_tests.rs"]
mod tests;
