//! What:
//!  Whether a watch opened before a clone's mount still sees the tree,
//!  and the sentence that names the watches a person must reopen.
//! Why:
//!  An inotify watch belongs to the directory object it was set on and not to
//!  the path,
//! so a mount over the slot leaves every watch opened before it on
//!  the covered directory,
//! where no write of the clone's own tree arrives
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Dependency repair and watchers").
//!  The comparison is by object and not by name:
//!  the device,
//! the inode and the birth time together say which directory a
//!  path names,
//! and an inode number alone is reused once its object goes.
//!  The sentence is the wrapper's whole part of the repair:
//!  an editor's watcher and Git's fsmonitor daemon are theirs to restart,
//!  and the owner decided on 2026-11-03 that the wrapper neither starts nor
//!  stops the monitor (`package/git-policy/cli/README.md`,
//!  "The wrapper's own Git and fsmonitor").
//!  No watch of the wrapper's own can be stale this way,
//! because each wait sets
//!  its watch when it starts waiting (`landing/service_index_watch.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const held = openThePath(slot); const watches = sameObject(recorded, held) ? 'same' : 'blind';
//! ```

use crate::fs_beneath::{
    ObjectId,
    identity_of,
    open_root,
};
use crate::shell_quote::shell_quote;
use std::path::Path;

/// What:
///  What a watch set before the mount still sees of the worktree.
/// Why:
///  The answer is the one the two objects give,
///  so it never rests on a name that two mounts can swap under it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Watches = 'still-the-same-tree' | 'blind';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Watches {
    /// The slot names the object the registration recorded,
    /// so a watch set before the mount still sees this tree.
    StillTheSameTree,
    /// The slot names another directory object,
    /// so a watch set before the mount sees the covered directory and no write
    /// in the clone's tree.
    Blind,
}

/// What:
///  Everything the wiring step reads once the clone is mounted.
/// Why:
///  The decision and the words are one answer,
///  so no caller can print a sentence its own comparison did not reach.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AfterTheMount = { watches: Watches; line?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AfterTheMount {
    /// What a watch set before the mount still sees.
    pub watches: Watches,
    /// The sentence for the person,
    /// present only when the watches are blind.
    pub line: Option<String>,
}

/// What:
///  The object the slot's path names now.
/// Why:
///  The comparison needs the object the path shows,
///  and nothing else about the slot is read:
///  a mount is proved by what the path names,
///  not by what the mount table or the handler says about it.
///  The path must be absolute,
/// because the open refuses to guess a directory
///  from this process's own.
///
/// # Errors
/// The path cannot be opened without following its last link,
/// or the identity of the directory it names cannot be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theObjectThePathShows(slot: string): ObjectId;
/// ```
pub fn the_object_the_path_shows(slot: &Path) -> std::io::Result<ObjectId> {
    let held: std::fs::File = open_root(slot)?;
    return identity_of(&held);
}

/// What:
///  What a watch set on the recorded object still sees once the path names
///  another.
/// Why:
///  The two readings are of one directory object when device,
/// inode and birth
///  time all agree,
///  and only then may a watch from before the mount still see the tree.
///  Every other pair is blind,
///  including two objects that share an inode number,
///  which a filesystem reuses once the first object goes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whatAWatchSetBeforeStillSees(before: ObjectId, now: ObjectId): Watches;
/// ```
pub fn what_a_watch_set_before_still_sees(
    before: ObjectId,
    now: ObjectId,
) -> Watches {
    return if before == now {
        Watches::StillTheSameTree
    } else {
        Watches::Blind
    };
}

/// What:
///  The sentence naming the watches a person must reopen after a mount.
/// Why:
///  It names the slot and the two programs whose watches are blind,
///  says what each one is,
///  and gives the check and the command that restarts the monitor,
///  so a reader needs no other page to act.
///  It also says that nothing of the wrapper's is left watching the covered
///  directory,
///  because a person who reads of one stale watch will wonder about the rest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function thePersonsLine(slot: string): string;
/// ```
pub fn the_persons_line(slot: &Path) -> String {
    let named: String = shell_quote(slot.as_os_str().as_encoded_bytes());
    return format!(
        "monochromatic-git-policy-cli: {named} now names the clone's own root, and the directory Git registered there is covered by that mount, so a file watch opened before the mount sees no write in the new tree. That is an editor's file watcher and Git's own fsmonitor daemon for this worktree. Ways forward: reopen this folder in the editor, or restart its file watching; and in the worktree run `git fsmonitor--daemon status`, then `git fsmonitor--daemon stop` if a monitor runs, because the next Git command starts a fresh one. Nothing the wrapper holds is left watching the covered directory: each of its waits sets its own watch when it starts.\n"
    );
}

/// What:
///  The decision and the sentence the wiring step reads after the mount.
/// Why:
///  It is the one call a later step makes:
///  the recorded object is the one the registration took before the mount,
///  and what the path shows now decides between silence and the sentence.
///
/// # Errors
/// The slot's path cannot be read as the directory it names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function afterTheMount(slot: string, recorded: ObjectId): AfterTheMount;
/// ```
pub fn after_the_mount(
    slot: &Path,
    recorded: ObjectId,
) -> std::io::Result<AfterTheMount> {
    let now: ObjectId = the_object_the_path_shows(slot)?;
    let watches: Watches = what_a_watch_set_before_still_sees(recorded, now);
    let line: Option<String> = match watches {
        Watches::StillTheSameTree => None,
        Watches::Blind => Some(the_persons_line(slot)),
    };
    return Ok(AfterTheMount {
        watches,
        line,
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "watch_rebinding_tests.rs"]
mod tests;
