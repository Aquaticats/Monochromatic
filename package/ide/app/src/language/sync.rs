//! What the interface thread tells the worker about the displayed document.

/// The stamp names the displayed file generation and revision.
use super::identity::DocumentStamp;
/// An accepted external reload supplies both texts and the edit list between them.
use crate::document::Reload;
/// What:
///  `Rope` is Helix's character-indexed text buffer;
///  `ChangeSet` is its list of edits from
///       one text to another.
/// Why:
///  Servers that take incremental changes are sent exactly these edits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type Rope, type ChangeSet } from 'helix-core';
/// ```
use helix_core::{ChangeSet, Rope};
/// What:
///  `PathBuf` is an owned filesystem path (sibling:
///  borrowed `&Path`).
/// Why:
///  The command travels to another thread and must own its data.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PathBuf = string;
/// ```
use std::path::PathBuf;

/// A file was displayed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DocumentOpen = { path: string; text: Rope; stamp: DocumentStamp };
/// ```
#[derive(Clone, Debug)]
pub struct DocumentOpen {
    /// Resolved absolute path of the displayed file,
    ///  as `Workspace::resolve` returns it.
    pub path: PathBuf,
    /// The displayed text;
    ///  a rope clone shares its chunks instead of copying characters.
    pub text: Rope,
    /// File generation and revision of that text.
    pub stamp: DocumentStamp,
}

/// An external reload of the displayed file was accepted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DocumentReload = { file: number; baseRevision: number; revision: number;
///                         previous: Rope; text: Rope; changes: ChangeSet };
/// ```
#[derive(Clone, Debug)]
pub struct DocumentReload {
    /// File generation of the displayed file.
    ///  `u64` is an unsigned 64-bit counter.
    pub file: u64,
    /// Revision the reload was computed from.
    pub base_revision: u64,
    /// Revision of the new text.
    pub revision: u64,
    /// Text of the base revision.
    pub previous: Rope,
    /// The new text,
    ///  which equals the file on disk.
    pub text: Rope,
    /// Edits from `previous` to `text`,
    ///  in character offsets.
    pub changes: ChangeSet,
}

/// Construction from the application's own reload record.
impl DocumentReload {
    /// What:
    ///  Copy what servers need out of a prepared reload.
    ///  `&Reload` lends the record,
    ///  so this
    ///       must run before `Document::apply_reload` consumes it;
    ///  send the result only when
    ///       that call returned true.
    /// Why:
    ///  The new revision is the base plus one,
    ///  exactly as `Document::apply_reload` counts.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static fromReload(file: number, reload: Reload): DocumentReload
    /// ```
    pub fn from_reload(file: u64, reload: &Reload) -> Self {
        return Self {
            file,
            base_revision: reload.base_revision(),
            revision: reload.base_revision() + 1,
            // `clone` on a rope copies a small handle; `clone` on the change set copies its edit list.
            previous: reload.previous_text().clone(),
            text: reload.text().clone(),
            changes: reload.changes().clone(),
        };
    }
}
