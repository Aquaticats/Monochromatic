//! Lazy,
//!  filesystem-free tree presentation over read-only directory snapshots.

/// What the rule files said about one directory's entries travels with its listing.
use crate::git_ignore::IgnoreAnswer;
/// Directory metadata retains native filenames and the order of the listing it was given.
use crate::workspace::DirectoryEntry;
/// Invalid tree operations report affected paths instead of silently changing unrelated rows.
use anyhow::{Result, bail};
/// What:
///  Maps own cached snapshots;
///  sets own expansion state;
///  Path borrows names and PathBuf owns them.
/// Why:
///  Unlike a recursive node graph,
///  these let background results update one directory independently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directories = new Map<string, DirectoryEntry[]>();
/// const expanded = new Set<string>();
/// ```
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::Arc,
};

/// Validate and replace individual directory snapshots without performing I/O.
mod listing;
/// Opaque request identity prevents stale directory replies from overwriting newer snapshots.
mod requests;
/// Flatten only visible entries and identify missing expanded-directory snapshots.
mod rows;

/// One visible row keeps its native path rather than reconstructing it from a display label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeRow {
    /// Original dirent metadata;
    ///  symbolic links retain their non-directory classification.
    pub entry: DirectoryEntry,
    /// What:
    ///  usize indexes nested rows;
    ///  unlike u32/u64/i32 it matches collection lengths.
    /// Why:
    ///  Indentation is derived from the visible traversal,
    ///  not lossy path-string parsing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// depth: number;
    /// ```
    pub depth: usize,
    /// Expanded directories may still be waiting for their first background snapshot.
    pub expanded: bool,
    /// What:
    ///  git would ignore this entry,
    ///  so its name is drawn dimmed;
    ///  the answer comes with the listing that contains the entry.
    /// Why:
    ///  A row that sits inside an ignored directory is ignored too;
    ///  a tracked file that matches a pattern is also reported ignored,
    ///  because the index is not read.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// ignored: boolean;
    /// ```
    pub ignored: bool,
}

/// What:
///  Arc shares an immutable path allocation across the UI and worker,
///  unlike thread-local Rc.
/// Why:
///  Allocation identity distinguishes repeated reads without a counter that can wrap or be reused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DirectoryRequest = { readonly identity: object; readonly path: string };
/// ```
#[derive(Clone, Debug)]
pub struct DirectoryRequest {
    /// Private allocation identity and native path cannot be rewritten by a reply consumer.
    path: Arc<PathBuf>,
}

/// Cached snapshots and expansion state for a single canonical Workspace root.
#[derive(Debug)]
pub struct FileTree {
    /// Canonical root supplied by Workspace;
    ///  this model is not an OS write-confinement boundary.
    root: PathBuf,
    /// Native directory names map to snapshots retaining listing order.
    directories: BTreeMap<PathBuf, Vec<DirectoryEntry>>,
    /// Collapsing an ancestor retains descendant expansion until that subtree is removed.
    expanded: BTreeSet<PathBuf>,
    /// Latest request for each directory;
    ///  replacements supersede older in-flight copies.
    pending: BTreeMap<PathBuf, DirectoryRequest>,
    /// What the rule files said when each directory was last listed;
    ///  a directory replaced by a newer snapshot takes its answer with it.
    ignores: BTreeMap<PathBuf, IgnoreAnswer>,
    /// Set when a listing arrived whose directory's own `.gitignore`,
    ///  or whose work tree's shared rules,
    ///  differ from the previous listing's;
    ///  the owner then rereads every shown directory and clears the flag with `take_ignore_sources_changed`.
    ignore_sources_changed: bool,
}

/// Tree state transitions never read files or block the UI on directory enumeration.
impl FileTree {
    /// Start without reading the root;
    ///  missing_listings requests its first background snapshot.
    pub fn new(root: &Path) -> Self {
        // What: to_path_buf copies the borrowed path into model-owned storage.
        // Why: The tree outlives the caller's temporary path borrow.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const ownedRoot = root;
        // ```
        let owned_root = root.to_path_buf();
        // What: clone duplicates the owned name; from constructs a set from a fixed one-item array.
        // Why: The implicit root is always expanded, without adding a redundant visible root row.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const expanded = new Set([ownedRoot]);
        // ```
        let expanded = BTreeSet::from([owned_root.clone()]);
        // What: new creates an empty map rather than reading or preloading descendants.
        // Why: Startup and each later expansion remain lazy.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { root: ownedRoot, directories: new Map(), expanded };
        // ```
        return Self {
            root: owned_root,
            directories: BTreeMap::new(),
            expanded,
            pending: BTreeMap::new(),
            ignores: BTreeMap::new(),
            ignore_sources_changed: false,
        };
    }

    /// Distinguish an empty loaded directory from one whose initial read has never succeeded.
    pub fn has_listing(&self, directory: &Path) -> bool {
        return self.directories.contains_key(directory);
    }

    /// Borrow a known entry from its parent's snapshot,
    ///  including collapsed descendants.
    fn entry(&self, path: &Path) -> Option<&DirectoryEntry> {
        // What: ? returns None when the optional parent or snapshot is absent.
        // Why: An unknown row is distinct from a known non-directory row.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return directories.get(parent(path))?.find(entry => entry.path === path);
        // ```
        let parent = path.parent()?;
        let entries = self.directories.get(parent)?;
        // Each entry is lent in turn; the first match is returned borrowed, without copying its name.
        for entry in entries.iter() {
            if entry.path == path {
                return Some(entry);
            }
        }
        return None;
    }

    /// Whether a listing since the last call showed changed ignore rules,
    ///  clearing the flag:
    ///  the caller then lists every shown directory again,
    ///  because a changed `.gitignore` can change the rows of any directory below it.
    pub fn take_ignore_sources_changed(&mut self) -> bool {
        return std::mem::take(&mut self.ignore_sources_changed);
    }

    /// Toggle a known directory;
    ///  files and symbolic links cannot become expandable tree nodes.
    pub fn toggle(&mut self, path: &Path) -> Result<bool> {
        // What: let-else extracts Some(entry) or exits through a descriptive error.
        // Why: Stale UI row identifiers cannot accidentally create new expansion state.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const entry = this.entry(path);
        // if (entry === undefined) throw new Error(`Unknown directory ${path}`);
        // ```
        let Some(entry) = self.entry(path) else {
            bail!("Cannot expand unknown tree entry {}", path.display());
        };
        if !entry.is_directory {
            bail!(
                "Cannot expand {}: its directory entry is not a directory",
                path.display()
            );
        }
        if self.expanded.remove(path) {
            tracing::debug!(path = %path.display(), "collapsed tree directory");
            // What: Ok carries a successful result; Err would carry a failure.
            // Why: False identifies a valid collapse, not a failed operation.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return false;
            // ```
            return Ok(false);
        }
        // Own the expansion key beyond this call's borrowed path lifetime.
        self.expanded.insert(path.to_path_buf());
        tracing::debug!(path = %path.display(), "expanded tree directory");
        // Report a successful expansion; callers schedule its missing snapshot separately.
        return Ok(true);
    }
}
