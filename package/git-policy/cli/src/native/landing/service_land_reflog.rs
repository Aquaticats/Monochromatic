//! What:
//!  The reflog message a landing writes when it moves a branch:
//! the kind of commit,
//! Git's word for that kind,
//! and the first line of the commit's message.
//! Why:
//!  The branch move is `git update-ref --stdin`,
//! which writes the message it is given and nothing else;
//!  a move without one left every row of `git reflog` blank,
//! and `git reflog` is the recovery tool a commit lost to a `reset` is found with
//!  (owner: "never discard user data").
//!  Native Git words the message after the commit it makes (`builtin/commit.c`):
//! `commit: <first line>`,
//!  `commit (initial):`,
//! `commit (amend):`,
//! `commit (merge):`,
//! `commit (cherry-pick):` and
//! `commit (revert):`,
//!  and copies the message's first line with its trailing newline (`update_head_with_reflog`,
//! `sequencer.c`).
//!  The message is one argument of `git update-ref -m`,
//! so no byte of the commit message can change the grammar of the transaction on standard input.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const subject = message.slice(0, message.indexOf('\n')); updateRef(['-m', `${action}: ${subject}`, '--stdin'], transaction);
//! ```

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The kinds of commit a landing moves a branch for,
/// each with the word native Git writes for it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Committed {
    /// An ordinary commit.
    Plain,
    /// The first commit of a branch that did not exist.
    Initial,
    /// `--amend`.
    Amend,
    /// A commit that concludes a merge (`MERGE_HEAD`).
    Merge,
    /// A commit that concludes a cherry-pick (`CHERRY_PICK_HEAD`).
    CherryPick,
    /// A commit that concludes a revert (`REVERT_HEAD`).
    Revert,
}

impl Committed {
    /// What:
    ///  The word native Git writes before the message's first line,
    ///  which is the `reflog_msg` of `builtin/commit.c`.
    /// Why:
    ///   The row a person reads in `git reflog` has to say the same thing after the same command.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const action = { plain: 'commit', initial: 'commit (initial)', amend: 'commit (amend)', merge: 'commit (merge)', cherryPick: 'commit (cherry-pick)', revert: 'commit (revert)' }[kind];
    /// ```
    pub fn action(self) -> &'static str {
        return match self {
            Self::Plain => "commit",
            Self::Initial => "commit (initial)",
            Self::Amend => "commit (amend)",
            Self::Merge => "commit (merge)",
            Self::CherryPick => "commit (cherry-pick)",
            Self::Revert => "commit (revert)",
        };
    }
}

/// What:
///  The batch's copy of the conclusion state,
/// when the batch concludes a merge,
/// a cherry-pick or a revert.
/// Why:
///   The files the worktree held are copied into the journal before the branch moves
/// (`conclusion_state_copy.rs`),
/// so the kind of conclusion is read there and not from a state the client may already have removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const conclusion = cleanup ? pathOf(journal, seq) : undefined;
/// ```
pub fn conclusion_copy(journal: &Path, seq: u64, cleanup: bool) -> Option<PathBuf> {
    if !cleanup {
        return None;
    }
    return Some(crate::conclusion_state_copy::path_of(journal, seq));
}

/// What:
///  Whether the conclusion copy holds the state file `name`.
/// Why:
///   The three state files name the three conclusions,
/// and a copy without one is not that conclusion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holds = (copy, name) => existsSync(join(copy, 'shadow', name));
/// ```
fn copy_holds(copy: Option<&Path>, name: &str) -> bool {
    let Some(folder) = copy else { return false };
    return folder.join("shadow").join(name).exists();
}

/// What:
///  The kind of commit a landing concludes,
/// from the requests' flag,
/// the branch's first commit,
/// the commit's parents,
/// and the conclusion state the batch removes.
/// Why:
///  Native Git decides the same way:
/// a branch with no head is `initial`,
///  `--amend` is `amend`,
/// `MERGE_HEAD` is `merge`,
/// and the sequencer's head files are the other two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kind = initial ? 'initial' : amend ? 'amend' : holds(copy, 'MERGE_HEAD') ? 'merge' : holds(copy, 'CHERRY_PICK_HEAD') ? 'cherryPick' : holds(copy, 'REVERT_HEAD') ? 'revert' : parents > 1 ? 'merge' : 'plain';
/// ```
pub fn kind(amend: bool, initial: bool, parents: usize, copy: Option<&Path>) -> Committed {
    if initial {
        return Committed::Initial;
    }
    if amend {
        return Committed::Amend;
    }
    if copy_holds(copy, "MERGE_HEAD") || parents > 1 {
        return Committed::Merge;
    }
    if copy_holds(copy, "CHERRY_PICK_HEAD") {
        return Committed::CherryPick;
    }
    if copy_holds(copy, "REVERT_HEAD") {
        return Committed::Revert;
    }
    return Committed::Plain;
}

/// What:
///  The first line of a commit message,
/// as native Git copies it into the reflog.
/// Why:
///   `update_head_with_reflog` copies the bytes up to and including the first newline,
/// so the row holds the message's first line and never its body;
/// a NUL cannot travel in an argument and ends the line as Git's own readers do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const firstLine = (message) => message.subarray(0, Math.min(...[message.indexOf(10), message.indexOf(0)].filter((at) => at >= 0)));
/// ```
fn first_line(message: &[u8]) -> &[u8] {
    let end: usize = message
        .iter()
        .position(|byte| { return *byte == b'\n' || *byte == 0; })
        .unwrap_or(message.len());
    return &message[..end];
}

/// What:
///  The message `git update-ref -m` carries for a commit of `kind` whose message is `message`.
/// Why:
///   The action and the first line are the whole of the row native Git writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reflogMessage = (kind, message) => `${kind.action()}: ${firstLine(message).toString()}`;
/// ```
pub fn message(kind: Committed, message: &[u8]) -> OsString {
    let subject: &[u8] = first_line(message);
    let action: &str = kind.action();
    let mut text: Vec<u8> = Vec::with_capacity(action.len() + 2 + subject.len());
    text.extend_from_slice(action.as_bytes());
    text.extend_from_slice(b": ");
    text.extend_from_slice(subject);
    return crate::path_bytes::os_from_path_bytes(text.as_slice());
}

/// Controls for this module stay out of the release executable.
#[cfg(test)]
#[path = "service_land_reflog_tests.rs"]
mod tests;
