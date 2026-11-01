//! Iterate expanded snapshots without recursion or filesystem reads.

/// Parent-owned snapshots and public visible-row values are shared by native UI and tests.
use super::{FileTree, TreeRow};
/// Pending directory requests retain exact native paths for the background reader.
use std::path::PathBuf;

/// Visible traversal and lazy loading have the same expansion boundary.
impl FileTree {
    /// Flatten visible rows in listing order;
    ///  the implicit root is not a duplicate visible row.
    pub fn rows(&self) -> Vec<TreeRow> {
        // What: Vec owns a growable list; unlike a fixed array its size follows expanded snapshots.
        // Why: Rebuilding this flat view needs no recursive node ownership or toolkit-specific state.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const rows: TreeRow[] = [];
        // const pending: Array<{entry: DirectoryEntry, depth: number}> = [];
        // ```
        let mut rows = Vec::new();
        let mut pending = Vec::new();
        // What: if-let extracts a present root snapshot while lending it rather than copying it.
        // Why: Before the first background reply, the visible tree is empty rather than fabricated.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const entries = directories.get(root);
        // if (entries !== undefined) pushInReverse(pending, entries, 0);
        // ```
        if let Some(entries) = self.directories.get(&self.root) {
            for entry in entries.iter().rev() {
                pending.push((entry, 0));
            }
        }
        // What: while-let pops (entry, depth) tuples until the optional stack result is absent.
        // Why: Reverse pushes preserve listing order while avoiding recursion on deep directory trees.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // while (pending.length) { const { entry, depth } = pending.pop(); }
        // ```
        while let Some((entry, depth)) = pending.pop() {
            let expanded = entry.is_directory && self.expanded.contains(&entry.path);
            // What: clone duplicates the row's owned native metadata, not the entire cached subtree.
            // Why: A caller can retain visible rows independently of later directory replacements.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // rows.push({ entry: { ...entry }, depth, expanded });
            // ```
            // The answer for a directory's entries is stored under that directory's path.
            let mut ignored = false;
            if let Some(parent) = entry.path.parent()
                && let Some(answer) = self.ignores.get(parent)
            {
                ignored = answer.names.contains(&entry.name);
            }
            rows.push(TreeRow {
                entry: entry.clone(),
                depth,
                expanded,
                ignored,
            });
            if expanded {
                // Borrow a loaded snapshot; a still-missing one has no visible children yet.
                if let Some(children) = self.directories.get(&entry.path) {
                    for child in children.iter().rev() {
                        pending.push((child, depth + 1));
                    }
                }
            }
        }
        return rows;
    }

    /// Request only expanded directories reachable through currently visible snapshots.
    pub fn missing_listings(&self) -> Vec<PathBuf> {
        if !self.directories.contains_key(&self.root) {
            if self.pending.contains_key(&self.root) {
                // The root read already occupies a request slot, so do not enqueue it again.
                return Vec::new();
            }
            // Copy the root into one owned request instead of returning a reference to mutable tree state.
            return vec![self.root.clone()];
        }
        // Collect native paths without issuing I/O from the presentation model.
        let mut missing = Vec::new();
        for row in self.rows() {
            if row.expanded
                && !self.directories.contains_key(&row.entry.path)
                && !self.pending.contains_key(&row.entry.path)
            {
                missing.push(row.entry.path);
            }
        }
        return missing;
    }
}
