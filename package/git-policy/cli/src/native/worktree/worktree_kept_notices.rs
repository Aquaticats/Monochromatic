//! What:
//!  The record of a worktree a removal kept,
//!  and the rule for when a person reads it:
//!  the next wrapper command in the repository names it once,
//!  with the path,
//!  the branch and the reason.
//! Why:
//!  A landed sweep hands a removal to the commit service and returns,
//! so the proof that the worktree may go is made again there,
//!  immediately before the destruction,
//!  and one of those readings may fail long after the merge printed its own line.
//!  The worktree then stays,
//! and the command that would have made the decision is over:
//! without a record the person would never learn that their worktree is still there,
//  or why,
//  which is the silence the sweep's own notice exists to prevent.
//!  The record lives in the common Git directory because the worktree it names is still registered,
//  and a person may well never run anything in it again:
//! every worktree's command shows it,
//  once,
//  before it runs.
//!  It is written before the removal's answer is settled,
//  so a service killed in between leaves a statement rather than silence.
//!  A record is removed only once its words have been written out,
//!  and only while the file still holds the text that was worded,
//!  exactly as a push failure notice is
//!  (`push_failure_notice_shown_then_removed.rs`),
//!  and a command whose output a program reads shows none and takes none.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await recordKeptWorktree(commonDir, { worktree, branch, why });
//! for (const notice of keptWorktreeNotices(commonDir)) notice.show(stderr);
//! ```

use crate::kept_names_in_the_git_directories::path_for_reading;
use crate::private_storage::{create_private_directory_all, write_private_file_unsynced};
use crate::push_failure_notice::fnv_hex;
use crate::push_failure_notice_claimed_by_one_command::{ClaimToken, claim, claimed_by_a_live_command, record_of_claim, take_over};
use crate::push_failure_notice_shown_then_removed::Notice;
use crate::shell_quote::shell_quote;
use std::path::{Path, PathBuf};

/// Directory under the common Git directory that holds the worktrees a removal kept.
pub const KEPT_NOTICE_DIRECTORY: &str = "monochromatic-git-policy-cli/kept-worktree-notices";

/// First line of every record,
/// which tells it from anything else written there.
const MAGIC: &str = "monochromatic-git-policy-cli kept worktree 1";

/// The line that names the worktree the record is about.
const WORKTREE_FIELD: &str = "worktree: ";

/// The line that names the branch it had checked out.
const BRANCH_FIELD: &str = "branch: ";

/// The end of a record's own name,
/// which the claim names of `push_failure_notice_claimed_by_one_command` recognise.
const RECORD_SUFFIX: &str = ".txt";

/// The prefix every line this program prints carries.
const PREFIX: &str = "monochromatic-git-policy-cli: ";

/// What:
///  The directory that holds the records of a repository.
/// Why:
///   A reader visits a Git directory this process may not have settled,
///  so the folder is spelled through the reader of kept names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directory = noticeDirectory(commonDir);
/// ```
pub fn kept_notice_directory(common_dir: &Path) -> PathBuf {
    return path_for_reading(common_dir, KEPT_NOTICE_DIRECTORY);
}

/// What:
///  The file one worktree's record is written under.
/// Why:
///  A second removal of the same worktree replaces its record rather than adding a second line,
/// and a path is what the record is about,
/// so the name is the path's digest:
/// two worktrees never share one,
/// and the name is one no person has to read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = `kept-${fnvHex(path)}${RECORD_SUFFIX}`;
/// ```
fn record_name(worktree: &Path) -> String {
    return format!("kept-{}{RECORD_SUFFIX}", fnv_hex(worktree.display().to_string().as_str()));
}

/// What:
///  What the person is told about one worktree a removal kept.
/// Why:
///   The line names the worktree,
///  the branch,
///  the reason,
///  and the one command that removes it,
///  because a person who expected the worktree to be gone has to be able to act on it at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = `${PREFIX}the commit service left the worktree ${worktree} of branch ${branch} in place after its branch landed: ${why}. Remove it with ...`
/// ```
fn words_of(worktree: &Path, branch: &str, why: &str) -> String {
    let quoted: String = shell_quote(worktree.as_os_str().as_encoded_bytes());
    return format!(
        "{PREFIX}the commit service left the worktree {} of branch {} in place though its branch landed: {}. Remove it with `git worktree remove {quoted}` when nothing there is yours.",
        worktree.display(),
        branch,
        trim_reason(why)
    );
}

/// The reason without the trailing sentence break a caller may have given it.
fn trim_reason(why: &str) -> &str {
    return why.trim_end_matches(['.', ' ']);
}

/// What:
///  Write the record that this worktree was kept,
///  with its branch and the reason.
/// Why:
///   The words are made when the notice is shown,
/// so the record holds what was decided and not a sentence about a repository that has moved on,
///  and a write that fails says so,
///  because the caller logs it and the person is otherwise never told.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordKeptWorktree(commonDir, { worktree, branch, why });
/// ```
///
/// # Errors
///
/// The words for a directory or a record that cannot be written.
pub fn record_kept_worktree(common_dir: &Path, worktree: &Path, branch: &str, why: &str) -> Result<(), String> {
    let directory: PathBuf = kept_notice_directory(common_dir);
    if let Err(error) = create_private_directory_all(directory.as_path()) {
        return Err(format!("creating the notice directory failed: {error}"));
    }
    let path: PathBuf = directory.join(record_name(worktree));
    let text: String = format!("{MAGIC}\n{WORKTREE_FIELD}{}\n{BRANCH_FIELD}{branch}\n\n{why}\n", worktree.display());
    let _removed = std::fs::remove_file(path.as_path());
    return match write_private_file_unsynced(path.as_path(), text.as_bytes()) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!("writing the notice failed: {error}")),
    };
}

/// One record as a command holds it before it shows it.
struct Record {
    /// The record's own name,
    /// where a notice whose words could not be written goes back.
    record: PathBuf,
    /// The worktree the record is about.
    worktree: PathBuf,
    /// The branch it had checked out,
    /// as the record holds it.
    branch: String,
    /// The reason it was kept.
    why: String,
}

/// What:
///  The record's parts,
///  or nothing when the file is not one this release wrote.
/// Why:
///   A file that is not a record says nothing to a person,
///  and a reader that guessed would print another program's text as its own sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parsed = parseRecord(text); // {worktree, branch, why} | undefined
/// ```
fn parse_record(text: &str) -> Option<(PathBuf, String, String)> {
    let mut lines = text.splitn(5, '\n');
    if lines.next()? != MAGIC {
        return None;
    }
    let worktree: &str = lines.next()?.strip_prefix(WORKTREE_FIELD)?;
    let branch: &str = lines.next()?.strip_prefix(BRANCH_FIELD)?;
    if !lines.next()?.is_empty() {
        return None;
    }
    let why: &str = lines.next()?.trim_end_matches('\n');
    if why.is_empty() {
        return None;
    }
    return Some((PathBuf::from(worktree), branch.to_owned(), why.to_owned()));
}

/// What:
///  The records this repository holds,
///  oldest name first,
///  each claimed by this command so only one command shows it.
/// Why:
///   A record is shown once;
///  two commands that read it before either showed it must not both print it,
///  and the claim is the rename only one of them can win.
///  A claim its command abandoned is taken over,
///  so a killed command loses nothing.
///  Nothing is removed here:
///  the removal waits until the words are out,
///  which only the caller knows.
///  A directory that is not there is one failed look,
///  and it is the usual case.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notices = keptWorktreeNotices(commonDir); // Notice[], each claimed
/// ```
pub fn kept_worktree_notices(common_dir: &Path) -> Vec<Notice> {
    let Ok(listing) = std::fs::read_dir(kept_notice_directory(common_dir)) else { return Vec::new() };
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
        let Some(claimed) = claim(path.as_path(), &token) else { continue };
        if let Some(notice) = notice_at(claimed.as_path()) {
            notices.push(notice);
        }
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
    let text: String = std::fs::read_to_string(claimed).ok()?;
    let (worktree, branch, why) = parse_record(text.as_str())?;
    let record: PathBuf = claimed.with_file_name(record_of_claim(claimed.file_name()?.to_string_lossy().as_ref())?);
    return Some(Notice { record, claim: claimed.to_path_buf(), text: words_of(worktree.as_path(), branch.as_str(), why.as_str()) });
}

/// Notice controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_kept_notices_tests.rs"]
mod tests;
