//! What:
//!  The choice the in-process index reset records and the trait behind it:
//! the worktree's stat data where a caller can prove the file holds the entry's bytes,
//! or the all-zero value
//!  `git reset --no-refresh` writes.
//! Why:
//!   `index_reset.rs` is built wherever `unix` holds and the proof that reads the worktree is built on Linux
//! (`index_reset_worktree_stat.rs`),
//! so the type the reset takes and the trait it calls live here,
//! where a Unix build that is not
//!  Linux still compiles and simply has no source to name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type WorktreeStat = { recorded: (path: Uint8Array, blob: Uint8Array, mode: number) => StatData | undefined } | 'none';
//! ```

/// The nine fields an entry records,
/// which a source answers with.
use crate::index_stat::StatData;

/// What:
///  A source of the stat data an entry the reset makes records:
/// the worktree's own where the caller can prove the file holds the entry's bytes.
/// Why:
///   The reset holds the choice and the proof holds the reading,
/// so this trait names the one method between them and no closure
///  carries either side's state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface WorktreeStatSource { recorded(path: Uint8Array, blob: Uint8Array, mode: number): StatData | undefined }
/// ```
pub trait WorktreeStatSource {
    /// The stat data the file at `path` provably has,
    /// for the entry `mode` records the bytes of `blob`,
    /// or nothing when the file proves nothing.
    fn recorded(&self, path: &[u8], blob: &[u8], mode: u32) -> Option<StatData>;
}

/// What:
///  Where the stat data of an entry the reset makes comes from.
/// Why:
///   `git reset --no-refresh` writes every entry it makes with all-zero stat data,
/// and a landing that leaves the worktree's stat data
///  instead saves the next commit a refresh (`index_reset_worktree_stat.rs`);
/// a caller that can prove no file holds the committed bytes
///  still writes the zero value,
/// so this is a choice the caller states.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorktreeStat = { recorded: WorktreeStatSource } | 'none';
/// ```
pub enum WorktreeStat<'a> {
    /// The worktree's stat data for a path whose file provably holds the bytes the entry's object name records,
    /// from the source the caller names.
    Recorded(&'a dyn WorktreeStatSource),
    /// Nothing:
    /// every entry the reset makes carries the all-zero stat data,
    /// as `git reset --no-refresh` leaves it.
    None,
}
