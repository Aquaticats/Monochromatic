//! What:
//!  The record a removal leaves when it went around what it did not make,
//!  the words a later command shows for it once,
//!  and the reading that turns one back into those words.
//! Why:
//!  A removal that went around a volume runs in a command nobody reads:
//!  a merge's sweep,
//!  or the commit service acting on a hand-over.
//!  The worktree it names is gone and Git no longer lists it,
//!  so only a command of the repository can ever say what was left,
//!  which is why the record lives in the common Git directory,
//!  under a name that is the removed path's digest,
//!  and is claimed by one command so it is shown once.
//!  This file is one part of the module whose name it carries,
//!  split because that module was over the line limit:
//!  `worktree_removal_goes_around_a_foreign_volume.rs` holds the look for the volumes and the
//!  report of a removal that could not finish,
//!  `worktree_removal_goes_around_a_foreign_volume_reports.rs` holds the words a person reads,
//!  and this one holds the record and its reader.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await recordLeftInPlace(commonDir, { worktree, branch, left });
//! for (const notice of leftInPlaceNotices(commonDir)) notice.show(stderr);
//! ```

use crate::kept_names_in_the_git_directories::path_for_reading;
use crate::private_storage::{
    create_private_directory_all,
    write_private_file_unsynced,
};
use crate::push_failure_notice::fnv_hex;
use crate::push_failure_notice_claimed_by_one_command::{
    ClaimToken,
    claim,
    claimed_by_a_live_command,
    record_of_claim,
    take_over,
};
use crate::push_failure_notice_shown_then_removed::Notice;
use crate::subvolume_remove_walk::{
    A_MOUNT_POINT,
    ANOTHER_FILESYSTEM,
    Boundary,
};
use crate::worktree_removal_goes_around_a_foreign_volume_reports::{
    PREFIX,
    freeing_the_path,
    items,
};
use std::path::{
    Path,
    PathBuf,
};

/// Directory under the common Git directory that holds the worktrees a removal went around.
pub const LEFT_IN_PLACE_NOTICE_DIRECTORY: &str =
    "monochromatic-git-policy-cli/left-in-place-notices";

/// First line of every record,
/// which tells it from anything else written there.
const MAGIC: &str = "monochromatic-git-policy-cli left in place 1";

/// The line that names the worktree the record is about.
const WORKTREE_FIELD: &str = "worktree: ";

/// The line that names the branch it had checked out.
const BRANCH_FIELD: &str = "branch: ";

/// The line that names one volume that was left,
/// as what it is and where it stands,
/// separated by a tab.
const LEFT_FIELD: &str = "left: ";

/// The end of a record's own name,
/// which the claim names of `push_failure_notice_claimed_by_one_command` recognise.
const RECORD_SUFFIX: &str = ".txt";

/// What:
///  The words a later command shows for one worktree a removal went around,
///  once.
/// Why:
///  A landed removal runs inside a merge,
/// and the person who asked for the merge may never see the sweep's output,
///  while the
///  path they want next is still taken.
/// The record is where that statement lives,
/// and it names the worktree,
/// its branch,
///  what was left and what frees the path,
/// exactly as the sweep's own line does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = leftInPlaceWords('/repo/wt', 'topic', left);
/// ```
fn left_in_place_words(
    worktree: &Path,
    branch: &str,
    left: &[Boundary],
) -> String {
    let branch_words: String = match branch.is_empty() {
        true => String::new(),
        false => format!(" of branch {branch}"),
    };
    return format!(
        "{PREFIX}the worktree {worktree}{branch_words} was removed after its branch landed, and Git no longer lists it, but what \
         someone else put inside it is left where it stands, so the path is still taken:\n{items}\nNothing of what is left was \
         moved, unmounted or deleted.\n{freeing}",
        worktree = worktree.display(),
        branch_words = branch_words,
        items = items(left),
        freeing = freeing_the_path(worktree)
    );
}

/// What:
///  The directory that holds the records of a repository.
/// Why:
///   A reader visits a Git directory this process may not have settled,
///  so the directory is spelled through the reader of kept names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directory = leftInPlaceDirectory(commonDir);
/// ```
pub fn left_in_place_directory(common_dir: &Path) -> PathBuf {
    return path_for_reading(common_dir, LEFT_IN_PLACE_NOTICE_DIRECTORY);
}

/// What:
///  The file one worktree's record is written under.
/// Why:
///  A second removal of the same worktree replaces its record rather than adding a second one,
/// and a path is what the record is about,
/// so the name is the path's digest:
/// two worktrees never share one,
/// and the name is no name a person has to read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = `left-${fnvHex(path)}${RECORD_SUFFIX}`;
/// ```
pub(crate) fn record_name(worktree: &Path) -> String {
    return format!(
        "left-{}{RECORD_SUFFIX}",
        fnv_hex(worktree.display().to_string().as_str())
    );
}

/// What:
///  Record that a removal of `worktree` went around what it did not make,
///  so the next command in the repository says so once.
/// Why:
///   The removal happened in a command nobody reads:
/// a merge's sweep,
///  or the commit service acting on a hand-over.
/// The worktree it names is gone and Git no longer lists it,
/// so no later command in it would ever show
///  anything:
/// only a command of the repository can,
/// which is why the record lives in the common Git directory.
/// A record that cannot be written costs the person that line and nothing else,
/// because the removal itself has finished.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordLeftInPlace(commonDir, { worktree, branch, left });
/// ```
///
/// # Errors
///
/// The words for a directory or a record that could not be written,
/// which a caller logs and never fails a removal for.
pub fn record_left_in_place(
    common_dir: &Path,
    worktree: &Path,
    branch: &str,
    left: &[Boundary],
) -> Result<(), String> {
    let directory: PathBuf = left_in_place_directory(common_dir);
    if let Err(error) = create_private_directory_all(directory.as_path()) {
        return Err(format!(
            "the directory of the records of what a removal left cannot be made ({error})"
        ));
    }
    let mut text: String = format!(
        "{MAGIC}\n{WORKTREE_FIELD}{}\n{BRANCH_FIELD}{branch}\n",
        worktree.display()
    );
    for found in left {
        text.push_str(LEFT_FIELD);
        text.push_str(found.what);
        text.push('\t');
        text.push_str(found.path.display().to_string().as_str());
        text.push('\n');
    }
    let path: PathBuf = directory.join(record_name(worktree));
    // The record of a path that was removed twice is the newer removal's, and a write that replaced the older text by a
    // rename never leaves a name that reads as two statements.
    let _removed = std::fs::remove_file(path.as_path());
    return match write_private_file_unsynced(path.as_path(), text.as_bytes()) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!(
            "the record {} of what a removal left cannot be written ({error})",
            path.display()
        )),
    };
}

/// What:
///  What one record says:
///  the worktree,
///  its branch,
///  and every volume that was left.
/// Why:
///   The reader needs all three to word the line,
/// and a record whose fields are incomplete says nothing a person can act on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LeftRecord = { worktree: string; branch: string; left: Boundary[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
struct LeftRecord {
    /// The worktree the removal took.
    worktree: PathBuf,
    /// The branch it had checked out,
    /// which may be empty.
    branch: String,
    /// What the removal left,
    /// each with what it is.
    left: Vec<Boundary>,
}

/// What:
///  What one of this program's two words for a volume stands for,
///  or nothing when the word is not one of them.
/// Why:
///   The words in a record are this program's own,
/// and a record that names another kind is one this build does not read,
///  which keeps a kind a later build adds out of this one's hands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const what = knownWhat('a mount point'); // the word, or undefined
/// ```
fn known_what(word: &str) -> Option<&'static str> {
    for known in [ANOTHER_FILESYSTEM, A_MOUNT_POINT] {
        if known == word {
            return Some(known);
        }
    }
    return None;
}

/// What:
///  The record a text holds,
///  or nothing when it is no record of this kind.
/// Why:
///   A file in that directory that this build cannot read is left for the build that wrote it,
///  exactly as a
///  queue record is,
///  and a record whose first line is another line is another program's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = parseRecord(text);
/// ```
fn parse_record(text: &str) -> Option<LeftRecord> {
    let mut lines = text.lines();
    if lines.next()? != MAGIC {
        return None;
    }
    let worktree: PathBuf = PathBuf::from(lines.next()?.strip_prefix(WORKTREE_FIELD)?);
    let branch: String = String::from(lines.next()?.strip_prefix(BRANCH_FIELD)?);
    let mut left: Vec<Boundary> = Vec::new();
    for line in lines {
        let Some(rest) = line.strip_prefix(LEFT_FIELD) else {
            continue;
        };
        let (what, path) = rest.split_once('\t')?;
        left.push(Boundary {
            path: PathBuf::from(path),
            what: known_what(what)?,
        });
    }
    if left.is_empty() {
        return None;
    }
    return Some(LeftRecord {
        worktree,
        branch,
        left,
    });
}

/// What:
///  Every record of a removal that went around what it did not make,
///  each claimed by this command so only one shows it.
/// Why:
///   The next wrapper command of the repository shows each of them and takes it,
///  and a command whose output a program reads
///  shows none and takes none,
///  exactly as a kept-worktree record does.
///  A claim its command abandoned is taken over,
///  so a killed command loses nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// leftInPlaceNotices(commonDir) // Notice[]
/// ```
pub fn left_in_place_notices(common_dir: &Path) -> Vec<Notice> {
    let Ok(listing) = std::fs::read_dir(left_in_place_directory(common_dir)) else {
        return Vec::new();
    };
    let token: ClaimToken = ClaimToken::of_this_command();
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in listing.flatten() {
        paths.push(entry.path());
    }
    paths.sort();
    let mut notices: Vec<Notice> = Vec::new();
    for path in paths {
        let Some(name) = path
            .file_name()
            .map(|found| return found.to_string_lossy().into_owned())
        else {
            continue;
        };
        if record_of_claim(name.as_str()).is_some() {
            if claimed_by_a_live_command(name.as_str()) {
                continue;
            }
            let Some(claimed) = take_over(path.as_path(), &token) else {
                continue;
            };
            if let Some(notice) = notice_at(claimed.as_path()) {
                notices.push(notice);
            }
            continue;
        }
        if !name.ends_with(RECORD_SUFFIX) {
            continue;
        }
        // The record is read before it is claimed, so a file this program did not write is neither shown nor moved:
        // renaming it first and finding it unreadable would take something of the person's out of its own name.
        // A name that holds a named pipe answers as unreadable at once rather than waiting for a writer.
        let Some(record) = crate::plain_file::read_plain_text(path.as_path())
            .ok()
            .and_then(|text| return parse_record(text.as_str()))
        else {
            continue;
        };
        let Some(claimed) = claim(path.as_path(), &token) else {
            continue;
        };
        let text: String = left_in_place_words(
            record.worktree.as_path(),
            record.branch.as_str(),
            record.left.as_slice(),
        );
        notices.push(Notice {
            record: path,
            claim: claimed,
            text,
        });
    }
    return notices;
}

/// What:
///  The notice one claimed record stands for,
///  or nothing when it is not a record this release wrote.
/// Why:
///   A claimed file nothing can read is left where it is:
///  deleting it would take away a statement nobody has read,
///  and it costs one look per command until a person removes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notice = noticeAt(claimed); // Notice | undefined
/// ```
fn notice_at(claimed: &Path) -> Option<Notice> {
    let read: String = crate::plain_file::read_plain_text(claimed).ok()?;
    let record: LeftRecord = parse_record(read.as_str())?;
    let original: PathBuf = claimed.with_file_name(record_of_claim(
        claimed.file_name()?.to_string_lossy().as_ref(),
    )?);
    let words: String = left_in_place_words(
        record.worktree.as_path(),
        record.branch.as_str(),
        record.left.as_slice(),
    );
    return Some(Notice {
        record: original,
        claim: claimed.to_path_buf(),
        text: words,
    });
}
