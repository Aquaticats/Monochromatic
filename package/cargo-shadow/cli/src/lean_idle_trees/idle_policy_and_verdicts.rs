//! What:
//! The idle periods a pass judges a tree by and what the pass says it did with each tree.
//! Why:
//! The decision, the removal and the pass all read these names,
//! and a reader of any one of them meets the types before the steps that use them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type IdlePolicy = { idleSeconds: number; unidentifiedIdleSeconds: number };
//! type TreeVerdict = { keep: string } | { skip: string } | { removed: { reason: string; bytes: number } } | { partlyRemoved: { reason: string; bytes: number; remaining: string } };
//! type TreeOutcome = { tree: string; verdict: TreeVerdict };
//! type TreeCollection = { outcomes: TreeOutcome[]; judged: number };
//! ```
/// Imports the owned path type the outcomes carry.
use std::path::PathBuf;
/// The idle periods of mechanism C.
#[derive(Clone, Copy, Debug)]
pub struct IdlePolicy {
    /// A tree whose workspace is gone (or whose toolchain is retired) is removed after this many seconds without a write.
    pub idle_seconds: i64,
    /// A tree nothing identifies is removed after this many seconds without a write.
    pub unidentified_idle_seconds: i64,
}

/// What the collection did with one tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TreeVerdict {
    /// Kept,
    /// for this reason.
    Keep(
        /// Why the tree is kept.
        String,
    ),
    /// Not judged or not removable now,
    /// for this reason.
    Skip(
        /// Why the tree was not judged or not removed.
        String,
    ),
    /// Removed (or in a dry run,
    /// would be) with these disk bytes.
    Removed {
        /// The reason it qualified.
        reason: String,
        /// Disk bytes freed or that would be.
        bytes: u64,
    },
    /// Part of it is gone and part of it stayed,
    /// because a directory the walk reached could not be emptied.
    PartlyRemoved {
        /// The reason it qualified.
        reason: String,
        /// Disk bytes its removal freed.
        bytes: u64,
        /// One reason for each part that stayed.
        remaining: String,
    },
}

/// One tree and what happened to it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreeOutcome {
    /// The tree.
    pub tree: PathBuf,
    /// What was done.
    pub verdict: TreeVerdict,
}

/// What one pass over a list of trees did.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TreeCollection {
    /// One outcome per tree that was judged,
    /// in the order given.
    pub outcomes: Vec<TreeOutcome>,
    /// How many of the trees it was given the pass judged before the deadline stopped it,
    /// which is the position the next run continues after.
    pub judged: usize,
}
