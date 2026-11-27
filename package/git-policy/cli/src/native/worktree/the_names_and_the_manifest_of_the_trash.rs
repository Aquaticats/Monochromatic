//! What:
//!  The names and the two records every trash file shares:
//! the directory a worktree's trash stands in,
//! the reference a trash work item carries,
//! the name of the file a trash writes about itself,
//! that manifest read back with the identity of the file it came from,
//! and the trash waiting for the commit service.
//! Why:
//!  Eight files name one or more of these,
//! and none of them moves a directory or empties a trash,
//! so they sit in a file of their own instead of in the three that do.
//! A name here is a plain string and a record here is a few fields,
//! so this file uses nothing of the folder and every trash file may list it first.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const TRASH_PARENT = '.claude/cli-git-trash';
//! const TRASH_REFERENCE = 'refs/cli-git/trash';
//! const MANIFEST_FILE_NAME = '.manifest';
//! ```

use crate::worktree_trash_manifest::TrashManifest;
use std::path::PathBuf;

/// The directory,
/// relative to a worktree root,
/// inside which each snapshot's trash directory is made.
/// It sits below `.claude/`,
/// which the repository ignores,
/// so a trash that has not been emptied yet is no untracked file.
pub const TRASH_PARENT: &str = ".claude/cli-git-trash";

/// The reference every trash work item carries;
/// the service accepts only a reference under `refs/`.
pub const TRASH_REFERENCE: &str = "refs/cli-git/trash";

/// What:
///  The name of the file a trash carries inside itself,
/// below its own name in a worktree.
/// Why:
///   The trash directory is the wrapper's own,
/// named with forty hexadecimal digits,
/// so one plain name inside it
///  cannot be mistaken for a directory the leave-out set aside,
/// which are all named `d<index>`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TRASH_MANIFEST_FILE = '.manifest';
/// ```
pub const MANIFEST_FILE_NAME: &str = ".manifest";

/// What:
///  The manifest a trash carries inside itself,
///  with the identity of the file that carries it.
/// Why:
///   The bytes say which trash the leave-out made,
/// and the identity of the file they were read through is what the emptying compares again
///  before it unlinks that file,
/// so a file another process moved onto the name is never removed unread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwnManifest = { manifest: TrashManifest; file: ObjectId };
/// ```
pub struct OwnManifest {
    /// What the leave-out made and moved.
    pub manifest: TrashManifest,
    /// The object that held those bytes when they were read.
    pub file: crate::fs_beneath::ObjectId,
}

/// What:
///  A snapshot's trash waiting for the commit service:
/// the worktree that holds it,
/// the repository's common
///  directory that its record is written under,
/// what the leave-out moved,
/// and whether the person asked for silence.
/// Why:
///   The add hands it over and returns;
/// the record is the common directory's,
/// because one repository's
///  service drains one work directory after another and empties one worktree's trashes one at a time,
/// whichever
///  worktree holds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PendingTrash = { destination: string; commonDir: string; manifest: TrashManifest; quiet: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingTrash {
    /// The new worktree that holds the trash.
    pub destination: PathBuf,
    /// The repository's common Git directory,
    /// whose work directory holds the record and whose service empties it.
    pub common_dir: PathBuf,
    /// What the leave-out made and moved.
    pub manifest: TrashManifest,
    /// How many of the trash's directories came from an earlier add's trash,
    /// which nothing of this add would ever
    ///  have emptied;
    /// the hand-over says so when it is more than none,
    /// because every add hands over at least the stand-ins and the
    ///  `.git` copy it made itself.
    pub taken_over: usize,
    /// Whether the person's `--quiet` silences the notices of the hand-over.
    pub quiet: bool,
}
