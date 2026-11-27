//! What:
//!  The two forms one step of a carry writes:
//!  what one path's index entry becomes,
//!  and what its worktree file becomes.
//! Why:
//!  The plans that decide them and the batch that writes them live in files of their own,
//!  and a step holds one of each,
//!  so the two are one file that names neither
//!  (`AGENTS.md` rule LYF:
//! a folder's files never use each other in a loop).
//!  Which step of a carry a batch is and what a started write learns are
//! `merge_isolation_carry_write_batches.rs` and `merge_isolation_carry_writes.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type IndexWrite = { kind: 'leave' } | { kind: 'entry'; mode: string; oid: string } | { kind: 'remove' };
//! ```

/// What one layer of one path writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexWrite = { kind: 'leave' } | { kind: 'entry'; mode: string; oid: string } | { kind: 'remove' };
/// ```
pub(crate) enum IndexWrite {
    /// The layer already holds it.
    Leave,
    /// The entry to write.
    Entry(String, String),
    /// Nothing stands at the path.
    Remove,
}

/// What the worktree of one path writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FileWrite = { kind: 'leave' } | { kind: 'bytes'; bytes: Uint8Array; mode: string; thePersonsBits?: number } | { kind: 'remove' };
/// ```
pub(crate) enum FileWrite {
    /// The layer already holds it.
    Leave,
    /// One entry's file form,
    /// the mode Git records for it,
    /// and the permission bits it is given when the mode is not the one a checkout of Git decides.
    Bytes {
        /// The bytes of the entry:
        /// a file's content or a link's target text.
        bytes: Vec<u8>,
        /// The mode Git records,
        /// which says the kind of the entry and whether its owner may run it.
        mode: String,
        /// Every permission bit the person's own entry carried,
        /// when the bytes are theirs and the merge wrote nothing at the path;
        /// nothing when Git's own checkout decides the mode of the file.
        the_persons_bits: Option<u32>,
    },
    /// Nothing stands at the path.
    Remove,
}
