//! What:
//!  What a pull kept of the person's untracked files,
//!  and what it could not keep:
//!  the copy of one file that stood where the branch move would write,
//!  the paths nothing was copied from and what stood there,
//!  and whether the untracked files could be listed at all.
//! Why:
//!  An empty answer is a promise that the reset Git offers discards nothing of the person's own,
//!  and a listing that could not run is no promise at all,
//!  so the answer says which of the two it is.
//!  The sentences of `pull_untracked_paths_in_the_way_words.rs` read these values,
//!  and so does the keeping itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type KeptPath = { inTheWorktree: string; inTheCopy: string };
//! type PathLeftInTheWay = { inTheWorktree: string; why: string };
//! type UntrackedInTheWay = { listingFailure?: string; kept: KeptPath[]; left: PathLeftInTheWay[]; folder?: string };
//! ```

use std::path::PathBuf;
/// What:
///  One file Git does not track,
/// where it stood in the worktree and where its copy is.
/// Why:
///   The person reads both in one sentence:
///  the path they know and the path they copy back from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type KeptPath = { inTheWorktree: string; inTheCopy: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeptPath {
    /// The path in the worktree,
    /// whose bytes the move would have overwritten.
    pub in_the_worktree: PathBuf,
    /// The copy of it,
    /// beside the Git directory.
    pub in_the_copy: PathBuf,
}

/// What:
///  One path the move would have overwritten that nothing was copied from,
///  and why.
/// Why:
///   The sentence offers the reset only after what stands there is moved away,
///  so the person has to know which paths those are and what stands there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PathLeftInTheWay = { inTheWorktree: string; why: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathLeftInTheWay {
    /// The path in the worktree.
    pub in_the_worktree: PathBuf,
    /// What stands there,
    /// or what stopped the copy.
    pub why: String,
}

/// What:
///  What the untracked files the move would have overwritten held,
///  and whether the wrapper could list them at all.
/// Why:
///   An empty answer is a promise that the reset discards nothing of the person's own;
///  a listing that could not run is no promise at all,
///  and the sentence says which it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UntrackedInTheWay = { listingFailure?: string; kept: KeptPath[]; left: PathLeftInTheWay[]; folder?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UntrackedInTheWay {
    /// Why the paths the move would overwrite could not be listed,
    /// if they could not.
    pub listing_failure: Option<String>,
    /// The files whose bytes are kept.
    pub kept: Vec<KeptPath>,
    /// The paths nothing was copied from.
    pub left: Vec<PathLeftInTheWay>,
    /// The folder the copies were made in,
    /// once one was made.
    pub folder: Option<PathBuf>,
}
