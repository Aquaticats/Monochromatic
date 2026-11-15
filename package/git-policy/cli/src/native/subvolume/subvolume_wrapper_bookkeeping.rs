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
//!  the notice folders and the first lines a notice may begin with come from the three modules that
//!  write and read notices,
//!  and the data folder from `wrapper_names_old_and_new`.
//!  A file under one of those names that this program did not write is a person's file and still counts:
//!  the file itself has to carry the mark its writer puts on it,
//!  the record header of the lock holders or one of the first lines the notice's own reader accepts,
//!  and a file without that mark is judged like any other entry.
//!  The kind an entry is judged by is the kind the walk listed,
//!  never a second look at the name:
//!  a name that came to hold something else while the pass ran is counted, not carried,
//!  so the deletion of the old tree is held back and the person looks at that path.
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
//!  - a kind this program writes without a mark its reader accepts on the first line,
//!    such as the work records the commit service lands and the logs beside them,
//!    is counted exactly as a person's entry is:
//!    nothing there tells this program's file from a person's of the same name,
//!    and stopping the deletion is the side that loses nothing.
//!    The data folder the notices stand in is carried so the walk reaches them;
//!    the folders beside it are not on that way and stay counted whole.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const own = ownBookkeepingOf(tree, '.git/cli-git-lock-holders/4242.0f3a', listed); // 'left-out' | 'carried-over' | undefined
//! ```

use crate::lock_holder_records::{RECORD_HEADER, folder_below_the_common_git_directory};
use crate::plain_file::open_plain_file;
use crate::push_failure_notice::{NOTICE_DIRECTORY, ORPHAN_NOTICE_DIRECTORY};
use crate::worktree_kept_notices::KEPT_NOTICE_DIRECTORY;
use crate::worktree_removal_goes_around_a_foreign_volume::LEFT_IN_PLACE_NOTICE_DIRECTORY;
use crate::wrapper_names_old_and_new::DATA_FOLDER;
use std::fs::Metadata;
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
/// type Told = 'record-header' | ((line: string) => boolean);
/// ```
#[derive(Clone, Copy)]
enum Told {
    /// The header a lock holder's record starts with,
    /// from the module that writes the records.
    RecordHeader,
    /// One of the first lines a notice module's reader accepts,
    /// from the module that writes and reads that notice.
    NoticeFirstLine(fn(&str) -> bool),
}

/// What:
///  One kind of notice this program shows a person:
///  the folders the module that writes it spells,
///  and the first line that module's reader accepts.
/// Why:
///   A judge of a notice file asks the writer for both,
///  so neither a folder nor a spelling is written down a second time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NoticeKind = { folders: string[]; accepts: (line: string) => boolean };
/// ```
pub(crate) struct NoticeKind {
    /// The folder of each notice of this kind,
    /// under both spellings the data folder has.
    pub(crate) folders: Vec<PathBuf>,
    /// Whether a first line is one this kind's reader accepts.
    pub(crate) accepts: fn(&str) -> bool,
    /// Every first line that reader accepts,
    /// which is what a control walks this kind through.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) first_lines: Vec<String>,
}

/// What:
///  What this program's own bookkeeping is at `relative`,
///  a path relative to `tree`,
///  or nothing when the entry there is a person's or Git's.
///  `listed` is the metadata the walk took when it listed the entry's folder.
/// Why:
///   See the module description.
/// The answer is read from the entry's own first line,
///  never from the folder's name alone,
///  so a person's file under a name this program keeps is still a person's,
///  and it is read for the kind the walk listed:
///  an entry that came to be another kind while the pass ran is counted, and holds the deletion back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const own = ownBookkeepingOf(oldTree, '.git/monochromatic-git-policy-cli/notices/x+0f3a', listed);
/// ```
pub(crate) fn own_bookkeeping_of(tree: &Path, relative: &Path, listed: &Metadata) -> Option<OwnBookkeeping> {
    if !tree.join(GIT_DIRECTORY).is_dir() {
        return None;
    }
    let below: &Path = relative.strip_prefix(GIT_DIRECTORY).ok()?;
    if below.as_os_str().is_empty() {
        return None;
    }
    let path: PathBuf = tree.join(relative);
    if below.starts_with(folder_below_the_common_git_directory()) {
        return decision(path.as_path(), listed, Told::RecordHeader);
    }
    for kind in notice_kinds() {
        if kind.folders.iter().any(|folder| return below.starts_with(folder)) {
            return decision(path.as_path(), listed, Told::NoticeFirstLine(kind.accepts));
        }
    }
    // The data folder itself is carried so the walk reaches the notices in it. A folder beside them,
    // such as the work records the commit service lands, is not on that way and stays counted like any other entry:
    // nothing here can tell this program's work record there from a person's file of the same name.
    for data_folder in [DATA_FOLDER.current, DATA_FOLDER.earlier] {
        if below == Path::new(data_folder) {
            return if listed.is_dir() { Some(OwnBookkeeping::CarriedOver) } else { None };
        }
    }
    return None;
}

/// What:
///  The decision for one entry below a folder this program writes into,
///  judged by the kind the walk listed and the mark its writer puts on the file.
/// Why:
///   A folder is carried over so the pass walks into it:
///  leaving a folder out would drop every file below it,
///  a person's included,
///  on the strength of the folder's name.
///  An entry the walk listed as a folder or a file that is not one of those when it is read is no entry this
///  judge knows,
///  so it is counted: the pass reports it and the deletion of the old tree waits for the person.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const decision = listed.isFolder ? 'carried-over' : (listed.isFile && marked(path, told) ? told.decision : undefined);
/// ```
fn decision(path: &Path, listed: &Metadata, told: Told) -> Option<OwnBookkeeping> {
    if listed.is_dir() {
        return Some(OwnBookkeeping::CarriedOver);
    }
    if !listed.is_file() {
        return None;
    }
    let first_line: String = first_line_of(path)?;
    let mine: bool = match told {
        Told::RecordHeader => first_line == RECORD_HEADER,
        Told::NoticeFirstLine(accepts) => accepts(first_line.as_str()),
    };
    if !mine {
        return None;
    }
    return Some(match told {
        Told::RecordHeader => OwnBookkeeping::LeftOut,
        Told::NoticeFirstLine(_) => OwnBookkeeping::CarriedOver,
    });
}

/// What:
///  The first line of the plain file at `path`,
///  or nothing when there is no plain file to read.
/// Why:
///   The mark is on the first line,
/// so a bounded read is all the answer needs;
///  a named pipe or a device at the name is refused by the open,
///  and a name that leads nowhere or holds a folder now is no mark either.
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
///  The kinds of notice this program writes,
///  each with the folders its module spells and the first lines its reader accepts,
///  under both spellings the data folder has.
/// Why:
///   The classifier holds no spelling of its own:
///  a module that accepts a first line there also names its folder here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kinds = [{ folders: ['monochromatic-git-policy-cli/notices', 'cli-git/notices', …], accepts: isPushNotice }];
/// ```
pub(crate) fn notice_kinds() -> Vec<NoticeKind> {
    let mut kinds: Vec<NoticeKind> = Vec::new();
    let mut push_folders: Vec<PathBuf> = Vec::new();
    for spelled in [NOTICE_DIRECTORY, ORPHAN_NOTICE_DIRECTORY] {
        push_folders.extend(spellings_of_the_data_folder(spelled));
    }
    kinds.push(NoticeKind { folders: push_folders, accepts: crate::push_failure_notice::is_a_notice_first_line, first_lines: crate::push_failure_notice::accepted_notice_first_lines() });
    kinds.push(NoticeKind {
        folders: spellings_of_the_data_folder(KEPT_NOTICE_DIRECTORY),
        accepts: crate::worktree_kept_notices::is_a_notice_first_line,
        first_lines: crate::worktree_kept_notices::accepted_notice_first_lines().iter().map(|line| return (*line).to_string()).collect(),
    });
    kinds.push(NoticeKind {
        folders: spellings_of_the_data_folder(LEFT_IN_PLACE_NOTICE_DIRECTORY),
        accepts: crate::worktree_removal_goes_around_a_foreign_volume::is_a_notice_first_line,
        first_lines: crate::worktree_removal_goes_around_a_foreign_volume::accepted_notice_first_lines().iter().map(|line| return (*line).to_string()).collect(),
    });
    return kinds;
}

/// What:
///  One folder a writer spells with the current data folder in front,
///  as a folder under each spelling the data folder has.
/// Why:
///   A repository renamed since an earlier release still holds notices under the earlier folder,
///  and `path_for_reading` reaches them there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const folders = spellingsOfTheDataFolder('monochromatic-git-policy-cli/notices');
/// ```
fn spellings_of_the_data_folder(spelled: &str) -> Vec<PathBuf> {
    let mut folders: Vec<PathBuf> = Vec::new();
    let Some(inside_the_data_folder) = spelled.strip_prefix(DATA_FOLDER.current) else { return vec![PathBuf::from(spelled)] };
    let below: &str = inside_the_data_folder.trim_start_matches('/');
    for data_folder in [DATA_FOLDER.current, DATA_FOLDER.earlier] {
        folders.push(Path::new(data_folder).join(below));
    }
    return folders;
}

/// Controls of the comparison stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_wrapper_bookkeeping_tests.rs"]
mod tests;
