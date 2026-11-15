//! What:
//!  Which entries under a checkout's Git directory are this program's own bookkeeping,
//!  and what a tree pass does with one that the destination lacks.
//! Why:
//!  The rule a migration's comparison follows:
//!  what it holds a migration to is what a person or Git put in the tree.
//!  A record this program writes for itself is this program's own,
//!  so it never stops a migration,
//!  and a person's file is never dropped without a word.
//!  The wrapper's bookkeeping is named here once,
//!  by the names the code that writes it uses,
//!  so no second list of paths can drift from those writers:
//!  the records folder comes from `lock_holder_records`,
//!  the notice folders from the three modules that write notices,
//!  and the program's own name,
//!  which every notice begins with,
//!  from `wrapper_names_old_and_new`.
//!  A file under one of those names that this program did not write is a person's file and still counts:
//!  the file itself has to carry the mark its writer puts on it,
//!  the record header of the lock holders or this program's name as the first word of a notice's first
//!  line,
//!  and a file without that mark is judged like any other entry.
//!  A folder under one of those names is never left out,
//!  because the folder's own name says nothing about the files below it:
//!  the pass walks into it and judges each file there on its own.
//!  What each kind gets, and why:
//!  - a lock holder's record is left out:
//!    it names processes that ran in the tree that is going away,
//!    it holds nothing a person wrote,
//!    and the checkout writes and sweeps such records itself,
//!    so carrying it back would only put litter into a live checkout.
//!  - a notice is carried over although it is older than the pass's cut-off:
//!    it is this program's word to the person about a push,
//!    a worktree a removal kept or a volume a removal went around,
//!    and a notice is removed from the checkout once it was shown,
//!    so the copy the old tree holds is shown once more rather than going silent with that tree.
//!  - a kind this program writes without a mark of its own on the first line,
//!    such as the work records the commit service lands and the logs beside them,
//!    is counted exactly as a person's entry is:
//!    nothing there tells this program's file from a person's of the same name,
//!    and stopping the deletion is the side that loses nothing.
//!    The data folder the notices stand in is carried so the walk reaches them;
//!    the folders beside it are not on that way and stay counted whole.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const own = ownBookkeepingOf(tree, '.git/cli-git-lock-holders/4242.0f3a'); // 'left-out' | 'carried-over' | undefined
//! ```

use crate::lock_holder_records::{RECORD_HEADER, folder_below_the_common_git_directory};
use crate::plain_file::open_plain_file;
use crate::push_failure_notice::{NOTICE_DIRECTORY, ORPHAN_NOTICE_DIRECTORY};
use crate::worktree_kept_notices::KEPT_NOTICE_DIRECTORY;
use crate::worktree_removal_goes_around_a_foreign_volume::LEFT_IN_PLACE_NOTICE_DIRECTORY;
use crate::wrapper_names_old_and_new::{DATA_FOLDER, PROGRAM};
use std::io::Read;
use std::path::{Path, PathBuf};

/// The Git directory of a main checkout,
/// relative to the checkout's root.
/// A checkout whose `.git` is a file has no entries of this program's beneath it,
/// and the migration refuses such a one before it copies anything.
const GIT_DIRECTORY: &str = ".git";

/// How many bytes of an entry are read to find the mark its writer put on it:
/// a header line and a notice's first line are each far shorter.
const MARK_BYTES: u64 = 1024;

/// What:
///  What a pass does with one entry of this program's own bookkeeping that the destination lacks and that is
///  older than the pass's cut-off.
/// Why:
///   The two kinds are kept for different reasons,
///  so the decision is named rather than a bare boolean,
///  and the pass counts each apart from a person's entries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwnBookkeeping = 'left-out' | 'carried-over';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OwnBookkeeping {
    /// This program's record of a process it started:
    /// left out,
    /// because it names that process and nothing a person wrote.
    LeftOut,
    /// This program's word to the person:
    /// carried over even though it is older than the cut-off,
    /// so what it says reaches the checkout.
    CarriedOver,
}

/// What:
///  The mark a writer puts on the first line of a file of one kind,
///  which is what tells the file from a person's file of the same name.
/// Why:
///   The name alone proves nothing:
///  a person may write a file under any name this program uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Mark = 'record-header' | 'program-name';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mark {
    /// The header a lock holder's record starts with,
    /// from the module that writes the records.
    RecordHeader,
    /// This program's own name,
    /// which is the first word of every notice it writes.
    ProgramName,
}

/// What:
///  What this program's own bookkeeping is at `relative`,
///  a path relative to `tree`,
///  or nothing when the entry there is a person's or Git's.
/// Why:
///   See the module description.
/// The answer is read from the entry itself,
///  never from the folder's name alone,
///  so a person's file under a name this program keeps is still a person's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const own = ownBookkeepingOf(oldTree, '.git/monochromatic-git-policy-cli/notices/x+0f3a');
/// ```
pub(crate) fn own_bookkeeping_of(tree: &Path, relative: &Path) -> Option<OwnBookkeeping> {
    if !tree.join(GIT_DIRECTORY).is_dir() {
        return None;
    }
    let below: &Path = relative.strip_prefix(GIT_DIRECTORY).ok()?;
    if below.as_os_str().is_empty() {
        return None;
    }
    let path: PathBuf = tree.join(relative);
    if below.starts_with(folder_below_the_common_git_directory()) {
        return decision(path.as_path(), Mark::RecordHeader);
    }
    for folder in notice_folders() {
        if below.starts_with(folder.as_path()) {
            return decision(path.as_path(), Mark::ProgramName);
        }
    }
    // The data folder itself is carried so the walk reaches the notices in it. A folder beside them,
    // such as the work records the commit service lands, is not on that way and stays counted like any other entry:
    // nothing here can tell this program's work record there from a person's file of the same name.
    for data_folder in [DATA_FOLDER.current, DATA_FOLDER.earlier] {
        if below == Path::new(data_folder) {
            return Some(OwnBookkeeping::CarriedOver);
        }
    }
    return None;
}

/// What:
///  The decision for one entry below a folder this program writes into,
///  by the mark its writer put on it.
/// Why:
///   A folder is carried over so the pass walks into it:
///  leaving a folder out would drop every file below it,
///  a person's included,
///  on the strength of the folder's name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const decision = marked(path) ? (mark === 'record-header' ? 'left-out' : 'carried-over') : undefined;
/// ```
fn decision(path: &Path, mark: Mark) -> Option<OwnBookkeeping> {
    let is_a_folder: bool = std::fs::symlink_metadata(path).map(|found| return found.is_dir()).unwrap_or(false);
    if is_a_folder {
        return Some(OwnBookkeeping::CarriedOver);
    }
    let first_line: String = first_line_of(path)?;
    let marked: bool = match mark {
        Mark::RecordHeader => first_line == RECORD_HEADER,
        Mark::ProgramName => names_the_program(first_line.as_str()),
    };
    if !marked {
        return None;
    }
    return Some(match mark {
        Mark::RecordHeader => OwnBookkeeping::LeftOut,
        Mark::ProgramName => OwnBookkeeping::CarriedOver,
    });
}

/// What:
///  The first line of the plain file at `path`,
///  or nothing when there is no plain file to read.
/// Why:
///   The mark is on the first line,
/// so a bounded read is all the answer needs;
///  a named pipe or a device at the name is refused by the open,
///  and a name that leads nowhere is no mark either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = readFirstLine(path); // undefined when it cannot be read
/// ```
fn first_line_of(path: &Path) -> Option<String> {
    let file: std::fs::File = open_plain_file(path).ok()?;
    let mut bytes: Vec<u8> = Vec::new();
    file.take(MARK_BYTES).read_to_end(&mut bytes).ok()?;
    let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(bytes.as_slice());
    let line: &str = text.lines().next()?;
    return Some(line.to_string());
}

/// What:
///  Whether a notice's first line names this program,
///  in the spelling this release writes or the one an earlier release wrote.
/// Why:
///   Every notice begins with the program's own name,
///  and a file whose first line names something else is no notice of this program's,
///  whatever folder it stands in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = (line) => ['monochromatic-git-policy-cli', 'cli-git-native'].includes(line.split(/[ \t]/)[0]);
/// ```
fn names_the_program(line: &str) -> bool {
    let name: &str = line.split([' ', '\t']).next().unwrap_or_default();
    return name == PROGRAM.current || name == PROGRAM.earlier;
}

/// What:
///  The folders below a checkout's Git directory that hold the notices this program shows a person,
///  each spelled by the module that writes it,
///  under both spellings the data folder has.
/// Why:
///   The writers spell the current data folder in front of their own folder,
///  and a repository renamed since an earlier release still holds notices under the earlier one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const folders = noticeFolders(); // ['monochromatic-git-policy-cli/notices', 'cli-git/notices', …]
/// ```
fn notice_folders() -> Vec<PathBuf> {
    let mut folders: Vec<PathBuf> = Vec::new();
    for spelled in [NOTICE_DIRECTORY, ORPHAN_NOTICE_DIRECTORY, KEPT_NOTICE_DIRECTORY, LEFT_IN_PLACE_NOTICE_DIRECTORY] {
        let Some(inside_the_data_folder) = spelled.strip_prefix(DATA_FOLDER.current) else { continue };
        let below: &str = inside_the_data_folder.trim_start_matches('/');
        for data_folder in [DATA_FOLDER.current, DATA_FOLDER.earlier] {
            folders.push(Path::new(data_folder).join(below));
        }
    }
    return folders;
}

/// Controls of the comparison stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_wrapper_bookkeeping_tests.rs"]
mod tests;
