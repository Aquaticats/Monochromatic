//! What:
//!  A three-way merge of text by line, and by word for the line regions that both sides changed, in process.
//!  A file is merged only when every such region resolves; the regions that needed the word pass are reported.
//! Why:
//!  Git's line merge calls two changes that touch (one ends where the other begins) a conflict, so an edit of a line plus
//!  lines inserted right after it stops a merge that nobody needs to judge. The owner chose, on 2026-11-03, a second pass
//!  in the way Jujutsu merges with `merge.hunk-level = "word"`.
//!  This is the red-control stub: it resolves nothing, so the controls that need a resolution fail.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const merged = mergeByWord({ base, ours, theirs }); // { kind: 'resolved', merged, parts } | { kind: 'conflict', unresolved }
//! ```

/// A run of lines of one text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LineSpan {
    /// The number of the first line, counting from 1; the line after the last one for a run with no lines.
    pub first: usize,
    /// How many lines the run holds.
    pub count: usize,
}

/// One line region that both sides changed and that the word pass merged.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MergedPart {
    /// The region in the common ancestor.
    pub base: LineSpan,
    /// The region in the first side.
    pub ours: LineSpan,
    /// The region in the second side.
    pub theirs: LineSpan,
    /// What the region became.
    pub merged: Vec<u8>,
}

/// How a three-way merge ended.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WordMerge {
    /// Every region resolved.
    Resolved {
        /// The whole merged text.
        merged: Vec<u8>,
        /// The regions that needed the word pass, in text order.
        parts: Vec<MergedPart>,
    },
    /// At least one region has two different changes to the same words.
    Conflict {
        /// How many line regions did not resolve.
        unresolved: usize,
    },
}

/// Merge `ours` and `theirs`, both derived from `base`; the stub never resolves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergeByWord(base: Uint8Array, ours: Uint8Array, theirs: Uint8Array): WordMerge;
/// ```
pub fn merge_by_word(_base: &[u8], _ours: &[u8], _theirs: &[u8]) -> WordMerge {
    return WordMerge::Conflict { unresolved: 0 };
}

/// Corpus equality, a random-edit differential and the properties of a merge.
#[cfg(test)]
#[path = "word_merge_tests.rs"]
mod tests;
