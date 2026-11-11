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
/// The index records one step writes,
/// whose entry the line about a deleted path reads.
use crate::merge_isolation_carry_writes::IndexWrite;
use crate::merge_isolation_changes::Change;
use crate::path_bytes::path_message_text;
/// The one owner of a path spelled for a shell,
/// so a command the person copies takes the path as one word.
use crate::shell_quote::shell_quote;

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

/// What stands in the index at a kept path,
/// of the person's own work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Staged = 'nothing' | 'the-paths-bytes' | 'the-merges-version' | 'own-version';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Staged {
    /// The index holds no entry for the path.
    Nothing,
    /// The index holds the bytes that stand at the path.
    ThePathsBytes,
    /// The index holds the merge's own version.
    TheMergesVersion,
    /// The index holds a version of the person's own,
    /// which a command that writes the path's bytes or the merge's version would replace.
    OwnVersion,
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
    /// What the index holds at the path after the carry,
    /// which decides whether the line may name a command that writes the index.
    pub staged: Staged,
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
/// const keptPath = (entry, change, why, staged) => ({ path: entry.path, why, worktree: holdsOf(entry), merge: change.existsAfter ? 'version' : 'nothing', staged });
/// ```
pub(crate) fn kept_path(entry: &CapturedPath, change: &Change, why: &'static str, staged: Staged) -> Kept {
    let merge: MergeHolds = if change.exists_after() { MergeHolds::Version } else { MergeHolds::Nothing };
    return Kept { path: entry.path.clone(), why, worktree: holds_of(entry), merge, staged };
}

/// The object of one index entry.
fn object_of(entry: &(String, String)) -> &str {
    return entry.1.as_str();
}

/// What:
///  The object the index holds for a kept path after the carry,
/// or nothing when it holds no entry.
/// Why:
///   A step writes its own index records,
/// and a step that leaves the index alone keeps what the reading found,
/// so only
///  these two say what the person will see there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const indexAfter = (index, ours, current) => ours ? (index.kind === 'entry' ? index.oid : index.kind === 'remove' ? null : current) : current;
/// ```
pub(crate) fn index_after<'a>(index: &'a IndexWrite, index_is_ours: bool, current: Option<&'a (String, String)>) -> Option<&'a str> {
    return match index {
        IndexWrite::Entry(_, oid) if index_is_ours => Some(oid.as_str()),
        IndexWrite::Remove if index_is_ours => None,
        _ => current.map(object_of),
    };
}

/// What:
///  What the index holds at a kept path,
/// told apart by the bytes each command in its line would write.
/// Why:
///   `git add` writes the bytes at the path and a restore can write the merge's version,
/// so a line may name one of
///  them only where it replaces nothing of the person's,
/// and `git add` over a staged version of theirs dropped that
///  version from the index
/// (`merge-notice-for-a-deleted-file` round 133,
/// finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stagedOf = (indexed, worktree, merge) => indexed === undefined ? 'nothing' : indexed === worktree ? 'the-paths-bytes' : indexed === merge ? 'the-merges-version' : 'own-version';
/// ```
pub(crate) fn staged_of(indexed: Option<&str>, worktree: Option<&str>, merge: Option<&str>) -> Staged {
    let Some(held) = indexed else { return Staged::Nothing };
    if Some(held) == worktree {
        return Staged::ThePathsBytes;
    }
    if Some(held) == merge {
        return Staged::TheMergesVersion;
    }
    return Staged::OwnVersion;
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

/// What:
///  The words of the commands that take the merge's deletion,
/// or say what keeps the person's side instead.
/// Why:
///   `git add` puts the bytes at the path in the index,
/// so it may be named only where the index holds no version of the
///  person's that it would replace;
/// a staged version and a later save are what stood after `git add` replaced one
///  with the other (`merge-notice-for-a-deleted-file` round 133,
/// finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const insteadWords = (path, holds, staged) => `${staged === 'nothing' ? `rm -- ${path}` : `git rm --force -- ${path}`} takes the merge's deletion, and ...`;
/// ```
fn instead_words(path: &str, holds: Holds, staged: Staged) -> String {
    // The wrapper's own worktree policy guards a forced `git rm` in a main worktree, so the command names the flag that
    //  policy asks for; `rm` is not Git and takes no flag.
    let take: String = if staged == Staged::Nothing {
        format!("`rm -- {path}`")
    } else {
        format!("`git rm --no-enforce-worktree --force -- {path}`")
    };
    let noun: &str = held_noun(holds);
    return match staged {
        // Nothing of the person's is in the index, so `git add` replaces nothing there; the merge's own version is no entry
        //  here, because it deletes the path.
        Staged::Nothing | Staged::TheMergesVersion => {
            format!("{take} takes the merge's deletion, and `git add -- {path}` keeps your {noun} in the index instead")
        }
        // The index holds the bytes at the path already, so staging them again says nothing new.
        Staged::ThePathsBytes => format!("{take} takes the merge's deletion, and the index already holds your {noun}"),
        // `git add` would put the bytes at the path in the index in place of the version the person staged there.
        Staged::OwnVersion => format!(
            "{take} takes the merge's deletion; leave the path as it is and both stand, because `git add -- {path}` would \
             replace the version the index holds with your {noun} at the path"
        ),
    };
}

/// What:
///  The words of the command that puts the merge's version of a kept path in the checkout,
/// and of what it leaves in the index.
/// Why:
///   A restore that names `--staged` writes the index too,
/// so it is named only where the index holds nothing of the
///  person's that it would replace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const restoreWords = (path, staged) => staged === 'own-version' ? `git restore --source=HEAD -- ${path} puts it in the checkout, and the version the index holds stays` : `git restore --source=HEAD --staged --worktree -- ${path} puts it in the checkout and the index`;
/// ```
fn restore_words(path: &str, staged: Staged) -> String {
    return match staged {
        // A version of the person's stands in the index, so the command writes the checkout alone.
        Staged::OwnVersion => format!(
            "`git restore --no-enforce-worktree --source=HEAD -- {path}` puts it in the checkout, and the version of yours that the \
             index holds stays there"
        ),
        // The index holds nothing of the person's here: a path whose bytes stood there holds nothing now, and a path the
        //  merge writes holds its own version at most.
        Staged::Nothing | Staged::ThePathsBytes | Staged::TheMergesVersion => format!(
            "`git restore --no-enforce-worktree --source=HEAD --staged --worktree -- {path}` puts it in the checkout and the index"
        ),
    };
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
///  Every path inside a command the line prints is quoted the way a shell reads a word back
/// (`crate::shell_quote`),
///  because a name with a space reached `rm` as three paths
/// (`merge-notice-for-a-deleted-file` round 133,
/// finding 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keptLine(reference: string, kept: Kept): string;
/// ```
pub(crate) fn kept_line(reference: &str, kept: &Kept) -> String {
    let path: String = path_message_text(kept.path.as_slice());
    let quoted: String = shell_quote(kept.path.as_slice());
    let named: String = format!("monochromatic-git-policy-cli: {reference}: {path}");
    let why: &str = kept.why;
    return match (kept.worktree, kept.merge) {
        // Nothing of the person's is at the path and the merge writes a file there: the words say what the capture saw and
        //  never how the path came to be empty, because a file a command set aside leaves the same state.
        (Holds::Nothing, MergeHolds::Version) => format!(
            "{named} holds nothing of yours in the checkout: {why}. The merge is made with nothing at {path}, and a deletion of \
             yours stands; the merge's version of {path} is in the merge commit, not in the checkout, and {}.",
            restore_words(quoted.as_str(), kept.staged)
        ),
        // Both sides hold a path the merge could not merge as text: the person's side stands whole and the merge's
        //  version waits in the commit the merge made.
        (Holds::File | Holds::Link, MergeHolds::Version) => format!(
            "{named} {}, because {why}. The merge's version of {path} is in the merge commit, not in the checkout; \
             `git restore --no-enforce-worktree --source=HEAD -- {quoted}` puts it in the checkout when you have read yours.",
            stands_words(kept.worktree)
        ),
        // The merge deletes the path and the person's side stands: their bytes are the only copy of it, and the merge's
        //  version is no entry at all.
        (Holds::File | Holds::Link, MergeHolds::Nothing) => format!(
            "{named} {}, because {why}. {}; {}.",
            stands_words(kept.worktree),
            merge_words(path.as_str(), kept.merge),
            instead_words(quoted.as_str(), kept.worktree, kept.staged)
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
