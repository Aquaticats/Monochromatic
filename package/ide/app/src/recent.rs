//! editord-compatible,
//!  session-local recent-file slots.

/// What:
///  Path and PathBuf represent borrowed and owned filesystem paths.
/// Why:
///  History owns names across file switches without assuming UTF-8 filenames.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Path = string;
/// ```
use std::path::{Path, PathBuf};

/// What:
///  A default-constructible record owns a growable path list.
/// Why:
///  Recently opened files are transient application state,
///  never a project file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RecentFiles { private paths: string[] = []; }
/// ```
#[derive(Default, Debug)]
pub struct RecentFiles {
    /// Most recent first,
    ///  with at most ten unique paths.
    paths: Vec<PathBuf>,
}

/// Operations preserve the paused editor's push-to-front semantics.
impl RecentFiles {
    /// Record a successful file open and promote existing entries.
    pub fn opened(&mut self, path: PathBuf) {
        // What: `drain(..)` moves every held path out one by one; those that differ from `path` are kept.
        // Why: Remove duplicates before inserting the new most-recent entry.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.paths = this.paths.filter(existing => existing !== path);
        // ```
        let mut kept: Vec<PathBuf> = Vec::new();
        for existing in self.paths.drain(..) {
            if existing != path {
                kept.push(existing);
            }
        }
        self.paths = kept;
        self.paths.insert(0, path);
        self.paths.truncate(10);
        tracing::debug!(count = self.paths.len(), "updated recent-file slots");
    }

    /// Borrow a slot without promoting it until opening succeeds.
    pub fn at(&self, index: usize) -> Option<&Path> {
        // What: get returns Some for an existing slot, None for an unfilled one.
        // Why: An empty Ctrl+digit slot is a normal no-op, not an error.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return this.paths[index];
        // ```
        if let Some(path) = self.paths.get(index) {
            // as_path lends Path rather than transferring the owned PathBuf.
            return Some(path.as_path());
        }
        return None;
    }

    /// Inspect slots for tree recency badges without exposing mutation.
    pub fn paths(&self) -> &[PathBuf] {
        // Borrow the existing list rather than clone it for every redraw.
        return &self.paths;
    }
}

/// Decode exactly Ctrl+0 through Ctrl+9,
///  excluding Shift and Alt variants.
pub fn shortcut_slot(key: &str, control: bool, shift: bool, alt: bool) -> Option<usize> {
    if !control || shift || alt || key.len() != 1 {
        return None;
    }
    // What: as_bytes lends UTF-8 bytes; length was checked before indexing.
    // Why: The shortcuts are specifically the ASCII digit keys.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const code = key.charCodeAt(0);
    // ```
    let code = key.as_bytes()[0];
    if !code.is_ascii_digit() {
        return None;
    }
    // What: b'0' is an ASCII byte and usize::from widens without truncation.
    // Why: Map displayed digits directly to the history's array indices.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return key.charCodeAt(0) - '0'.charCodeAt(0);
    // ```
    return Some(usize::from(code - b'0'));
}
