//! What:
//!  The removal that goes around a subvolume or a mount point it did not make:
//!  the words it says on the person's command,
//!  the line a landed sweep prints and the record a later command shows,
//!  and the line a later `git worktree add` adds to Git's own refusal.
//! Why:
//!  A worktree can hold a snapshot someone took there,
//! a container's storage or a mounted image,
//! and none of the removal's other routes may take it:
//! the kernel cannot destroy a subvolume that contains another one (`ENOTEMPTY`),
//! and Git's own removal descends into a mount point and deletes what is mounted there.
//! The project decided on 2026-11-12 ("Remove around it") that such a worktree is removed except for
//!  what was not the removal's:
//! the volume and the directories that lead to it stay where they stand,
//! Git forgets the worktree,
//! and the person is told what is left and what frees the path.
//! Both routes make the same decision:
//! the removal a person typed and the one a merge's sweep or the commit service performs.
//! The words live here because four callers say them,
//! and the line a later add prints must not disagree with the removal's own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const report = aroundRemovalReport(target, left);
//! const line = landedAroundLine(landed, worktree, branch, left);
//! await recordLeftInPlace(commonDir, worktree, branch, left);
//! for (const notice of leftInPlaceNotices(commonDir)) notice.show(stderr);
//! const addLine = addBesideAVolumeLine(destination, left);
//! ```

use crate::fs_beneath::open_root;
use crate::kept_names_in_the_git_directories::path_for_reading;
use crate::private_storage::{create_private_directory_all, write_private_file_unsynced};
use crate::push_failure_notice::fnv_hex;
use crate::push_failure_notice_claimed_by_one_command::{ClaimToken, claim, claimed_by_a_live_command, record_of_claim, take_over};
use crate::push_failure_notice_shown_then_removed::Notice;
use crate::shell_quote::shell_quote;
use crate::subvolume_remove_walk::{A_MOUNT_POINT, ANOTHER_FILESYSTEM, Boundary, BoundaryLook, look_for_boundaries};
use crate::worktree_removal_takes_its_pending_deletions::is_a_leaving_name;
use std::path::{Path, PathBuf};

/// The prefix every line of this program carries.
const PREFIX: &str = "monochromatic-git-policy-cli: ";

/// Directory under the common Git directory that holds the worktrees a removal went around.
pub const LEFT_IN_PLACE_NOTICE_DIRECTORY: &str = "monochromatic-git-policy-cli/left-in-place-notices";

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
///  The subvolumes and mount points a tree holds that its removal did not make,
///  and nothing when it may take the tree as it always did.
/// Why:
///   A tree holding one is never given to Git,
/// whose own removal deletes what is mounted at a mount point and unlinks the
///  files of a nested worktree;
/// this is the one look every route makes,
/// and it changes nothing it judges.
/// The names this
///  program leaves a subvolume under while its commit service deletes it are no reason to go around:
/// they hold
///  nothing of the person's,
/// and the removal moves them out before it destroys anything.
/// A tree that cannot be
///  looked at reports nothing here and is the caller's own decision,
/// because a rescue from a look that failed is not a proof.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const foreign = volumesBelow('/repo/wt'); // Boundary[]
/// ```
pub(crate) fn volumes_below(target: &Path) -> Vec<Boundary> {
    let Ok(root) = open_root(target) else { return Vec::new() };
    let look: BoundaryLook = look_for_boundaries(&root, target);
    let mut foreign: Vec<Boundary> = Vec::new();
    for found in look.found {
        if !is_a_leaving_name(found.path.as_path()) {
            foreign.push(found);
        }
    }
    return foreign;
}

/// What:
///  The words that give one volume and its path,
///  as a line of a list.
/// Why:
///   Every report names each volume the same way,
///  so a person who read one recognises the next.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = `  - ${what} at ${path}`;
/// ```
fn item(found: &Boundary) -> String {
    return format!("  - {what} at {path}", what = found.what, path = found.path.display());
}

/// What all of them are,
/// one line each.
fn items(left: &[Boundary]) -> String {
    let mut lines: Vec<String> = Vec::with_capacity(left.len());
    for found in left {
        lines.push(item(found));
    }
    return lines.join("\n");
}

/// The shell-quoted spelling of a path,
/// for a command a person copies.
fn quoted(path: &Path) -> String {
    return shell_quote(path.as_os_str().as_encoded_bytes());
}

/// What:
///  What frees the path once the person moves what was left out:
///  the `rmdir`s that take the empty directories,
///  and the one command that releases a volume of each kind.
/// Why:
///   A report that named only what was left would leave the person with a path nothing can be added at,
/// and the ways differ:
///  a mount point is released with `umount`,
///  which keeps what is mounted there;
/// a subvolume that is another repository's worktree is removed by that repository's own
///  command,
///  which keeps its branch.
/// `rmdir` is named first because it is the step that needs no decision about what a volume holds:
///  the directories it takes hold nothing by then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const freeing = freeingThePath('/repo/wt');
/// ```
fn freeing_the_path(path: &Path) -> String {
    return format!(
        "Move each one out of the directory it stands in. The directories that then hold nothing go from the deepest one up \
         with `rmdir`, which the kernel allows for an empty subvolume root too, and {quoted} is free for `git worktree add` \
         once they are gone. A mount point is released with `umount`, which keeps what is mounted there; a subvolume that is \
         another repository's worktree goes with `git worktree remove <path>` run in that repository, which keeps its branch \
         and refuses while anything there is uncommitted.",
        quoted = quoted(path)
    );
}

/// What:
///  The one report a person's own `git worktree remove` prints when the worktree went and something stayed.
/// Why:
///   The command did what it was asked:
/// the worktree's own files are gone and Git no longer lists it.
/// It has to say the rest in the same breath,
///  because the path is still taken and a person who reads only "removed" looks for a free path:
/// what was
///  left,
///  that nothing of it was moved or deleted,
///  and what frees the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = aroundRemovalReport('/repo/wt', left);
/// ```
pub fn around_removal_report(target: &Path, left: &[Boundary]) -> String {
    return format!(
        "{PREFIX}removed the worktree {target}, and Git no longer lists it, but what someone else put inside it is left where \
         it stands, so the path is still taken:\n{items}\nThe directories that lead to each of them are left too, and every \
         other entry of the worktree was removed. Nothing of what is left was moved, unmounted or deleted.\n{freeing}",
        target = target.display(),
        items = items(left),
        freeing = freeing_the_path(target)
    );
}

/// What:
///  The refusal of a person's removal that would have to delete more than Git does without `--force`,
///  while a volume someone else put there stands below it.
/// Why:
///   A tree holding such a volume is never given to Git,
/// whose own removal would delete what is mounted at one,
///  so Git's own refusal cannot be the answer here.
/// The report says that nothing was removed and why,
/// names the volumes once,
///  and gives both ways on:
/// `--force`, which removes the rest around them exactly as this command does with a clean tree,
/// and clearing or
///  moving what Git refuses to delete.
///  It never claims to be Git's sentence,
/// because the reason it refuses may be one Git does not judge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = refusedBesideAVolumeReport({ target, operand: '../wt', left, reason });
/// ```
pub fn refused_beside_a_volume_report(target: &Path, operand: &str, left: &[Boundary], reason: &str) -> String {
    return format!(
        "{PREFIX}nothing was removed, and nothing of what is named here was deleted. The worktree {target} holds what someone \
         else put inside it, so no removal of it may be handed to Git, whose own removal would delete what is mounted at a mount \
         point and cannot destroy a subvolume that contains one:\n{items}\nWhat Git refuses to delete without `--force` is there \
         too: {reason}.\n`git worktree remove --force {operand}` removes every other entry of the worktree and leaves what is \
         named above exactly as it is. To keep what Git would delete as well, move that out first and run this removal again; to \
         keep everything, move what is named above out and run it again.",
        target = target.display(),
        items = items(left),
        reason = reason,
        operand = operand
    );
}

/// What:
///  The report of a removal that removed what it could and stopped,
///  so the worktree is partly emptied and still registered.
/// Why:
///   A step that fails halfway leaves files gone and a registration standing,
/// and a person told either "removed" or
///  "nothing was removed" would believe the wrong one.
/// The report says which happened,
/// what stopped it,
///  and what finishes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = stoppedAroundReport(target, reason);
/// ```
pub fn stopped_around_report(target: &Path, reason: &str) -> String {
    return format!(
        "{PREFIX}removing the worktree {target} stopped before it finished: every entry it removed is gone, and the worktree is \
         still registered, so Git still lists it. What stopped it is: {reason}\nNothing that someone else put inside the \
         worktree was moved, unmounted or deleted. Fix what the reason names and run the same removal again; it goes on from \
         where this one stopped.",
        target = target.display(),
        reason = reason
    );
}

/// What:
///  The line a landed sweep prints for a worktree it removed and left something in.
/// Why:
///   The sweep removed a person's worktree without being asked,
/// so its line has to say that the worktree is gone from
///  Git's records, what stayed where it stands,
///  and what frees the path:
/// a person who read only "removed" would try to
///  add a worktree at a path that is still taken.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = landedAroundLine('main', '/repo/wt', 'topic', left);
/// ```
pub fn landed_around_line(landed_branch: &str, worktree: &Path, branch: &str, left: &[Boundary]) -> String {
    return format!(
        "{PREFIX}removed the worktree {worktree} by itself, because its branch {branch} is now on {landed}: every other entry of \
         it was removed and Git no longer lists it, and what someone else put inside it is left where it stands, so the path is \
         still taken:\n{items}\nThe branch stays. Nothing of what is left was moved, unmounted or deleted.\n{freeing}",
        worktree = worktree.display(),
        branch = branch,
        landed = landed_branch,
        items = items(left),
        freeing = freeing_the_path(worktree)
    );
}

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
fn left_in_place_words(worktree: &Path, branch: &str, left: &[Boundary]) -> String {
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
fn record_name(worktree: &Path) -> String {
    return format!("left-{}{RECORD_SUFFIX}", fnv_hex(worktree.display().to_string().as_str()));
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
pub fn record_left_in_place(common_dir: &Path, worktree: &Path, branch: &str, left: &[Boundary]) -> Result<(), String> {
    let directory: PathBuf = left_in_place_directory(common_dir);
    if let Err(error) = create_private_directory_all(directory.as_path()) {
        return Err(format!("the directory of the records of what a removal left cannot be made ({error})"));
    }
    let mut text: String = format!("{MAGIC}\n{WORKTREE_FIELD}{}\n{BRANCH_FIELD}{branch}\n", worktree.display());
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
        Err(error) => Err(format!("the record {} of what a removal left cannot be written ({error})", path.display())),
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
        let Some(rest) = line.strip_prefix(LEFT_FIELD) else { continue };
        let (what, path) = rest.split_once('\t')?;
        left.push(Boundary { path: PathBuf::from(path), what: known_what(what)? });
    }
    if left.is_empty() {
        return None;
    }
    return Some(LeftRecord { worktree, branch, left });
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
    let Ok(listing) = std::fs::read_dir(left_in_place_directory(common_dir)) else { return Vec::new() };
    let token: ClaimToken = ClaimToken::of_this_command();
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in listing.flatten() {
        paths.push(entry.path());
    }
    paths.sort();
    let mut notices: Vec<Notice> = Vec::new();
    for path in paths {
        let Some(name) = path.file_name().map(|found| return found.to_string_lossy().into_owned()) else { continue };
        if record_of_claim(name.as_str()).is_some() {
            if claimed_by_a_live_command(name.as_str()) {
                continue;
            }
            let Some(claimed) = take_over(path.as_path(), &token) else { continue };
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
        let Some(record) = std::fs::read_to_string(path.as_path()).ok().and_then(|text| return parse_record(text.as_str())) else { continue };
        let Some(claimed) = claim(path.as_path(), &token) else { continue };
        let text: String = left_in_place_words(record.worktree.as_path(), record.branch.as_str(), record.left.as_slice());
        notices.push(Notice { record: path, claim: claimed, text });
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
    let read: String = std::fs::read_to_string(claimed).ok()?;
    let record: LeftRecord = parse_record(read.as_str())?;
    let original: PathBuf = claimed.with_file_name(record_of_claim(claimed.file_name()?.to_string_lossy().as_ref())?);
    let words: String = left_in_place_words(record.worktree.as_path(), record.branch.as_str(), record.left.as_slice());
    return Some(Notice { record: original, claim: claimed.to_path_buf(), text: words });
}

/// What:
///  The line a `git worktree add` prints when the destination holds what a removal left,
///  or nothing when nothing is there.
/// Why:
///   Git's sentence for an occupied destination says only that the directory exists
/// (`fatal: <path> already exists`),
///  and the person who removed the worktree there was told a volume stayed.
/// The line names each volume and connects the two,
/// so that "already exists" is answered by what stands there,
///  and it comes from the same look the removal makes,
/// which is why the two cannot disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = addBesideAVolumeLine(destination, left);
/// ```
pub fn add_beside_a_volume_line(destination: &Path, left: &[Boundary]) -> String {
    return format!(
        "{PREFIX}the directory {destination} is not empty because a removal of the worktree there had to leave what someone else \
         put inside it:\n{items}\nNothing of it was moved or deleted. {freeing}",
        destination = destination.display(),
        items = items(left),
        freeing = freeing_the_path(destination)
    );
}

/// Words and records of a removal that went around a volume stay under the release's own readers.
#[cfg(test)]
#[path = "worktree_removal_goes_around_a_foreign_volume_tests.rs"]
mod tests;
