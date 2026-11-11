//! What:
//!  One path a carry kept whole:
//! what stands at it now of the person's own work,
//! what the merge made of it,
//! and
//!  the line that says both and names the command that reaches the merge's version.
//! Why:
//!   The kept path is the person's to judge,
//! and the line is the only thing that tells them where their work is and where
//!  the merge's is (`SPEC.md`,
//! "Merges and commits never fail",
//! "Carry edits forward";
//! the owner's rule "never discard user data").
//!  A path whose state differs from a plain file,
//! deleted,
//! a symbolic link,
//! a mode the word pass could not take,
//! is
//!  named by what stands there,
//! because a sentence that assumes a file sends the person to bytes that are not in it.
//!  The two states are carried in the entry rather than read back out of the reason:
//! the reason says why the merge could
//!  not take the path,
//! and only the capture and the change say what the path holds now.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = keptLine('feature', keptPath(entry, change, why, indexed));
//! ```

use crate::merge_isolation_carry_parts::CapturedPath;
use crate::merge_isolation_changes::Change;
use crate::path_bytes::path_message_text;

/// The Git mode of a symbolic link.
const SYMBOLIC_LINK_MODE: &str = "120000";

/// What stands at a kept path in the checkout,
/// of the person's own work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Holds = 'file' | 'link' | 'nothing';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Holds {
    /// The person's own bytes of a plain file.
    File,
    /// The person's own symbolic link,
    /// whose bytes are its target's text.
    Link,
    /// Nothing:
    /// the person deleted the path,
    /// and that deletion stands.
    Nothing,
}

/// What the merge makes of a kept path in the merge commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MergeHolds = 'version' | 'nothing';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MergeHolds {
    /// The merge's own version of the path,
    /// which `git restore --source=HEAD` puts back.
    Version,
    /// The merge deletes the path,
    /// so the merge commit holds no entry for it.
    Nothing,
}

/// One path a carry kept whole,
/// and the two states its line names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Kept = { path: Uint8Array; why: string; worktree: Holds; merge: MergeHolds; inTheIndex: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Kept {
    /// Path from the top of the worktree,
    /// exact bytes.
    pub path: Vec<u8>,
    /// Why the merge could not take the path,
    /// as the layer decided it.
    pub why: &'static str,
    /// What the person's side stands as in the checkout now.
    pub worktree: Holds,
    /// What the merge makes of the path in the merge commit.
    pub merge: MergeHolds,
    /// Whether the index holds an entry for the path after the carry,
    /// which `git rm` needs and `rm` must not be given.
    pub in_the_index: bool,
}

/// What:
///  The kind of entry a captured worktree file stands as.
/// Why:
///   The capture reads a link's target as its bytes,
/// so the mode it recorded is the only part of it that says the
///  path holds a link rather than a file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdsOf = (entry) => entry.worktree === null ? 'nothing' : entry.worktree.mode === '120000' ? 'link' : 'file';
/// ```
fn holds_of(entry: &CapturedPath) -> Holds {
    return match &entry.worktree {
        Some(file) if file.mode == SYMBOLIC_LINK_MODE => Holds::Link,
        Some(_) => Holds::File,
        None => Holds::Nothing,
    };
}

/// What:
///  The kept entry of one captured path,
/// with what stands at it and what the merge made of it.
/// Why:
///   The line is read by a person who must find both sides,
/// so the two states come from the capture and the change
///  rather than from the words of the reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const keptPath = (entry, change, why, inTheIndex) => ({ path: entry.path, why, worktree: holdsOf(entry), merge: change.existsAfter ? 'version' : 'nothing', inTheIndex });
/// ```
pub(crate) fn kept_path(entry: &CapturedPath, change: &Change, why: &'static str, in_the_index: bool) -> Kept {
    let merge: MergeHolds = if change.exists_after() { MergeHolds::Version } else { MergeHolds::Nothing };
    return Kept { path: entry.path.clone(), why, worktree: holds_of(entry), merge, in_the_index };
}

/// The clause that names what the person's side stands as.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const standsWords = (holds) => holds === 'file' ? 'holds your own bytes' : holds === 'link' ? 'is your own symbolic link' : 'holds nothing';
/// ```
fn stands_words(holds: Holds) -> &'static str {
    return match holds {
        Holds::File => "holds your own bytes",
        Holds::Link => "is your own symbolic link",
        Holds::Nothing => "holds nothing",
    };
}

/// The clause that names where the merge's version of the path is,
/// as the start of a sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mergeWords = (path, merge) => merge === 'version' ? `The merge's version of ${path} is in the merge commit, not in the checkout` : `The merge commit holds no ${path}`;
/// ```
fn merge_words(path: &str, merge: MergeHolds) -> String {
    return match merge {
        MergeHolds::Version => format!("The merge's version of {path} is in the merge commit, not in the checkout"),
        MergeHolds::Nothing => format!("The merge commit holds no {path}"),
    };
}

/// The noun the person's kept side is,
/// for the line that keeps it in the index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const heldNoun = (holds) => holds === 'link' ? 'link' : 'bytes';
/// ```
fn held_noun(holds: Holds) -> &'static str {
    return match holds {
        Holds::Link => "link",
        Holds::File | Holds::Nothing => "bytes",
    };
}

/// The words of the two commands that take the merge's deletion,
/// or keep the person's side instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const insteadWords = (path, holds, indexed) => `${indexed ? `git rm --force -- ${path}` : `rm -- ${path}`} takes the merge's deletion, and git add -- ${path} keeps your ${heldNoun(holds)} in the index instead`;
/// ```
fn instead_words(path: &str, holds: Holds, in_the_index: bool) -> String {
    // The wrapper's own worktree policy guards a forced `git rm` in a main worktree, so the command names the flag that
    //  policy asks for; `rm` is not Git and takes no flag.
    let take: String = if in_the_index {
        format!("`git rm --no-enforce-worktree --force -- {path}`")
    } else {
        format!("`rm -- {path}`")
    };
    let noun: &str = held_noun(holds);
    return format!("{take} takes the merge's deletion, and `git add -- {path}` keeps your {noun} in the index instead");
}

/// What:
///  The line one kept path adds to standard error.
/// Why:
///   Each line names the path,
/// what stands there of the person's work,
/// where the merge's version is,
/// and the
///  command that reaches it,
/// so a verdict about a binary file,
/// a link,
/// a deletion or a mode change is something the
///  person can act on.
///  A line never says a file holds the person's bytes when the path holds nothing,
/// and it never says the person
///  deleted a path the capture only saw empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keptLine(reference: string, kept: Kept): string;
/// ```
pub(crate) fn kept_line(reference: &str, kept: &Kept) -> String {
    let path: String = path_message_text(kept.path.as_slice());
    let named: String = format!("monochromatic-git-policy-cli: {reference}: {path}");
    let why: &str = kept.why;
    return match (kept.worktree, kept.merge) {
        // Nothing of the person's is at the path and the merge writes a file there: the words say what the capture saw and
        //  never how the path came to be empty, because a file a command set aside leaves the same state.
        (Holds::Nothing, MergeHolds::Version) => format!(
            "{named} holds nothing of yours: {why}. The merge is made with nothing at {path}, and a deletion of yours stands; the \
             merge's version of {path} is in the merge commit, not in the checkout, and `git restore --no-enforce-worktree \
             --source=HEAD --staged --worktree -- {path}` puts it in the checkout and the index."
        ),
        // Both sides hold a path the merge could not merge as text: the person's side stands whole and the merge's
        //  version waits in the commit the merge made.
        (Holds::File | Holds::Link, MergeHolds::Version) => format!(
            "{named} {}, because {why}. The merge's version of {path} is in the merge commit, not in the checkout; \
             `git restore --no-enforce-worktree --source=HEAD -- {path}` puts it in the checkout when you have read yours.",
            stands_words(kept.worktree)
        ),
        // The merge deletes the path and the person's side stands: their bytes are the only copy of it, and the merge's
        //  version is no entry at all.
        (Holds::File | Holds::Link, MergeHolds::Nothing) => format!(
            "{named} {}, because {why}. {}; {}.",
            stands_words(kept.worktree),
            merge_words(path.as_str(), kept.merge),
            instead_words(path.as_str(), kept.worktree, kept.in_the_index)
        ),
        // A path neither side holds has nothing to keep, which no decision of a layer makes; a line that names it must
        //  still be true.
        (Holds::Nothing, MergeHolds::Nothing) => format!(
            "{named} {}, because {why}. {}, so the checkout and the merge agree about it.",
            stands_words(kept.worktree),
            merge_words(path.as_str(), kept.merge)
        ),
    };
}

/// The kept path's own words.
#[cfg(test)]
#[path = "merge_isolation_carry_kept_path_tests.rs"]
mod tests;
