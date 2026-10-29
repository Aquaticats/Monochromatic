//! Validate complete background snapshots before replacing any visible tree state.

/// Parent-private state remains owned by the same tree rather than duplicated in a worker.
use super::FileTree;
/// The rule files' answer for the snapshot is installed with it or not at all.
use crate::git_ignore::IgnoreAnswer;
/// Preserve native names and dirent kinds from the existing read-only workspace reader.
use crate::workspace::DirectoryEntry;
/// Invalid snapshots are errors,
///  not partial updates or empty-directory fallbacks.
use anyhow::{Result, bail};
/// Sets detect duplicate paths without changing input order;
///  paths retain non-UTF-8 names.
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path, PathBuf},
};

/// What:
///  Whether `path` or any of its ancestors is in `removed`.
///  `Path::ancestors` yields the path itself first,
///  then each parent.
/// Why:
///  A cached path below a replaced directory describes a subtree that no longer exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const liesUnder = (path: string, removed: Set<string>) => ancestors(path).some((parent) => removed.has(parent));
/// ```
fn lies_under(path: &Path, removed: &BTreeSet<PathBuf>) -> bool {
    for parent in path.ancestors() {
        if removed.contains(parent) {
            return true;
        }
    }
    return false;
}

/// What:
///  Drop every entry of `cache` whose path lies under a removed directory.
///  The doomed paths are copied out first,
///  because a map cannot be changed while its keys are walked.
/// Why:
///  Prune each cache once,
///  without comparing every cached path with every removed directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const path of [...cache.keys()]) if (liesUnder(path, removed)) cache.delete(path);
/// ```
fn prune_map<Value>(cache: &mut BTreeMap<PathBuf, Value>, removed: &BTreeSet<PathBuf>) {
    let mut doomed: Vec<PathBuf> = Vec::new();
    for path in cache.keys() {
        if lies_under(path, removed) {
            doomed.push(path.clone());
        }
    }
    for path in doomed {
        cache.remove(&path);
    }
}

/// The same pruning for a set of paths.
fn prune_set(cache: &mut BTreeSet<PathBuf>, removed: &BTreeSet<PathBuf>) {
    let mut doomed: Vec<PathBuf> = Vec::new();
    for path in cache.iter() {
        if lies_under(path, removed) {
            doomed.push(path.clone());
        }
    }
    for path in doomed {
        cache.remove(&path);
    }
}

/// Check one snapshot independently so a rejected reply cannot partly update expansion state.
fn validate(directory: &Path, entries: &[DirectoryEntry]) -> Result<()> {
    // What: BTreeSet::new owns unique keys; Vec would permit duplicates and require repeated scans.
    // Why: Duplicate row paths must be rejected without sorting the displayed entries.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const seen = new Set<string>();
    // ```
    let mut seen = BTreeSet::new();
    for entry in entries {
        // What: components separates native path segments; Normal excludes '.', '..', roots and prefixes.
        // Why: A snapshot must contain immediate children, not traversal expressions or synthetic paths.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!isSingleNormalFilename(entry.name)) throw new Error('Invalid child');
        // ```
        let mut components = Path::new(&entry.name).components();
        let valid_name = matches!(components.next(), Some(Component::Normal(name)) if name == entry.name.as_os_str())
            && components.next().is_none();
        if !valid_name || entry.path != directory.join(&entry.name) {
            bail!(
                "Invalid tree entry {} in directory {}",
                entry.path.display(),
                directory.display()
            );
        }
        // Lend each path while validating; the set does not outlive this snapshot borrow.
        if !seen.insert(&entry.path) {
            bail!(
                "Duplicate tree entry {} in directory {}",
                entry.path.display(),
                directory.display()
            );
        }
    }
    // What: Ok(()) reports validation success without producing another copy of the snapshot.
    // Why: The caller can install its original ordered entries only after all checks pass.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return;
    // ```
    return Ok(());
}

/// Apply snapshots without performing I/O or treating the presentation model as security confinement.
impl FileTree {
    /// Replace a synchronous snapshot atomically;
    ///  asynchronous readers must use complete_listing's token check.
    /// Removed or reclassified subtrees lose stale caches and pending reads.
    /// The snapshot has no ignored entries,
    ///  as before an answer exists.
    pub fn apply_listing(&mut self, directory: &Path, entries: Vec<DirectoryEntry>) -> Result<()> {
        return self.apply_listing_with(directory, entries, IgnoreAnswer::default());
    }

    /// Replace a synchronous snapshot together with the answer of the rule files for its entries;
    ///  both are installed or neither is.
    /// A directory whose own `.gitignore`,
    ///  or whose work tree's shared rules,
    ///  differ from the previous snapshot's marks the tree for a reread of everything shown.
    pub fn apply_listing_with(
        &mut self,
        directory: &Path,
        entries: Vec<DirectoryEntry>,
        answer: IgnoreAnswer,
    ) -> Result<()> {
        if directory != self.root {
            // What: if-let extracts a borrowed entry when its parent snapshot still contains it.
            // Why: Replies for removed directories must not resurrect detached tree state.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const entry = this.entry(directory);
            // if (entry === undefined || !entry.isDirectory) throw new Error('Unknown directory');
            // ```
            if let Some(entry) = self.entry(directory) {
                if !entry.is_directory {
                    bail!(
                        "Cannot list non-directory tree entry {}",
                        directory.display()
                    );
                }
            } else {
                bail!("Cannot list unknown tree directory {}", directory.display());
            }
        }
        // What: ? propagates validation errors before the tree's first mutation.
        // Why: A malformed reply preserves both visible rows and expansion state.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // validate(directory, entries);
        // ```
        validate(directory, &entries)?;
        // Borrow keys of surviving directories once, avoiding a repeated scan per old entry.
        let mut surviving = BTreeSet::new();
        for entry in &entries {
            if entry.is_directory {
                surviving.insert(entry.path.as_path());
            }
        }
        // Gather owned detached names before mutably borrowing the same directory map.
        let mut removed = BTreeSet::new();
        // Borrow the previous snapshot if one exists; first loads have no detached subtrees.
        if let Some(previous) = self.directories.get(directory) {
            for old in previous {
                // A directory replaced with a file or symlink must also discard its descendants.
                if old.is_directory && !surviving.contains(old.path.as_path()) {
                    // What: clone copies the owned native path instead of borrowing the soon-replaced snapshot.
                    // Why: Cache pruning runs after the immutable snapshot borrow ends.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // removed.add(old.path);
                    // ```
                    removed.insert(old.path.clone());
                }
            }
        }
        // What: `prune_map` and `prune_set` keep only paths with no removed ancestor; ancestors includes the path itself.
        // Why: Prune each cache once, without comparing every cached path with every removed directory.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // keepPathsWhere(path => !ancestors(path).some(parent => removed.has(parent)));
        // ```
        prune_map(&mut self.directories, &removed);
        prune_set(&mut self.expanded, &removed);
        // Detached requests cannot become current again if the same directory name is later recreated.
        prune_map(&mut self.pending, &removed);
        // A synchronous snapshot also supersedes any older in-flight read of this directory.
        self.pending.remove(directory);
        prune_map(&mut self.ignores, &removed);
        // A first listing has no earlier rules to differ from; later ones compare stamps, not names.
        if let Some(previous) = self.ignores.get(directory)
            && previous.stamps != answer.stamps
        {
            tracing::debug!(path = %directory.display(), "the ignore rules of a directory changed; every shown directory will be listed again");
            self.ignore_sources_changed = true;
        }
        tracing::debug!(path = %directory.display(), entries = entries.len(), ignored = answer.names.len(), "applied tree directory snapshot");
        self.ignores.insert(directory.to_path_buf(), answer);
        // Copy only the directory key; move the already ordered entries into the cache.
        self.directories.insert(directory.to_path_buf(), entries);
        // Successful application does not return another snapshot copy.
        return Ok(());
    }
}
