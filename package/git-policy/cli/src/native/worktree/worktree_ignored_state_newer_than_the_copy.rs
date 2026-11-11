//! What:
//!  The ignored entries of a source that changed after the snapshot the add installed,
//!  the one line that says so,
//!  and the file that lists every one of them.
//! Why:
//!  An add usually installs a snapshot the commit service made ahead of it,
//!  because one `BTRFS_IOC_SNAP_CREATE_V2` on the add's own thread waits for a
//!  commit of the whole filesystem
//!  (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`).
//!  Such a copy is the source as of an earlier instant,
//!  so an ignored file written after it is in the new worktree as it was then,
//!  or is not there at all,
//!  and nothing in Git's own output says so.
//!  A person who meets a `node_modules` without the package they installed a minute ago is confused
//!  by it,
//!  so the add names every entry the copy is behind on,
//!  once,
//!  and lists all of them in the worktree's own Git directory when the first few do not fit one line.
//!  The source keeps every one of them,
//!  and the tool that writes them (a package install,
//!  a build) makes them again in the new worktree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const missed = entriesNewerThanTheCopy({ context, source, madeAt }); // the paths the copy is behind on
//! tellWhatTheCopyMisses({ context, source, destination, admin, madeAt, note }); // one line and the list file
//! ```

use crate::json_record::quote;
use crate::transaction_git::GitContext;
use crate::worktree_copy_ceiling::CopyCeiling;
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_copy_manifest::{EntryKind, LeftOut, WorktreeCopyEntry, collect_pruned_manifest, stamp_of};
use crate::worktree_copy_selection::{CopySelection, filesystem_path, select_roots, shown_repository_path};
use crate::worktree_copy_stage_leaf_copying::stamp_nanos;
use crate::worktree_snapshot_source::SnapshotSource;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// The name of the file,
/// in a worktree's Git directory,
/// that lists every ignored entry the snapshot the worktree received is behind on.
/// Why:
///   The one line on standard error names the first entries and the count;
/// a tree whose dependency folder was installed in the meantime can be behind on thousands,
///  and the person who needs one of them reads all of them there.
/// The name carries the program,
/// because the directory belongs to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LIST_NAME = 'monochromatic-git-policy-cli-ignored-state-after-the-copy.txt';
/// ```
pub const LIST_NAME: &str = "monochromatic-git-policy-cli-ignored-state-after-the-copy.txt";

/// How many paths the one line names before it says how many more there are.
/// Why:
///   A line naming a thousand paths is unreadable,
/// and the file beside the worktree's Git directory holds every one of them.
/// The cap is the one the messages for entries left out use (`NOTICE_PATHS`),
///  so one rule states how much text a notice names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NOTICE_PATHS = 3;
/// ```
const NOTICE_PATHS: usize = 3;

/// What:
///  The reason a copy step failed,
/// as the one sentence it holds.
/// Why:
///   The two steps that answer this module hand back the copy error,
/// and this module's caller reads a reason;
/// a named function says so once,
/// and a closure that captures nothing would say it twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reasonOf = (error: WorktreeCopyError) => error.message;
/// ```
fn reason_of(error: WorktreeCopyError) -> String {
    return error.0;
}

/// What:
///  The selection the copy of a source takes,
/// which is what decides the leave-out rule of the walk below.
/// Why:
///   The new worktree holds the entries a copy from this source takes,
/// so the question "what is the copy behind on" is asked over exactly those entries;
/// a linked worktree hands
///  over everything it ignores,
/// and the main checkout leaves its caches and run records out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const selection = source.gitDir === source.commonDir ? 'main-checkout' : 'ignored';
/// ```
fn selection_of(source: &SnapshotSource) -> CopySelection {
    return if source.git_dir == source.common_dir { CopySelection::MainCheckout } else { CopySelection::Ignored };
}

/// What:
///  The ignored files and symbolic links of the source that changed after the copy was made.
/// Why:
///   A change time in the future of the copy's instant proves a write after it,
/// and no program can set a change time,
///  so a file restored with an old modification time is still found.
/// A directory is not named:
/// it holds no bytes of its own,
///  and a change inside it is told by the files below it that changed,
/// which are the names a person compares.
///  The worktree this add made is not part of the source's ignored state and is left out,
/// and an entry the
///  source no longer holds says nothing,
/// because the walk lists what stands there now.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entriesNewerThanTheCopy({ context, source, madeAt }): Uint8Array[] { const roots = await selectRoots(...); const entries = walk(roots); return entries.filter((e) => ctimeOf(join(source.root, e)) > madeAt); }
/// ```
pub fn entries_newer_than_the_copy(context: &GitContext, source: &SnapshotSource, destination: &Path, made_at_millis: u128) -> Result<Vec<Vec<u8>>, String> {
    let selection: CopySelection = selection_of(source);
    let mut excluded: Vec<PathBuf> = Vec::new();
    for root in &source.nested {
        if root.starts_with(source.root.as_path()) {
            excluded.push(root.clone());
        }
    }
    // The worktree this add just made is not part of the source's ignored state: every file of it was written after the
    // copy, by this add, and naming a thousand of them would bury the entries the copy is really behind on.
    excluded.push(destination.to_path_buf());
    let roots: Vec<Vec<u8>> = select_roots(context, source.root.as_path(), excluded.as_slice(), &selection).map_err(reason_of)?;
    let mut left_out: LeftOut = LeftOut::default();
    let entries: Vec<WorktreeCopyEntry> = collect_pruned_manifest(
        source.root.as_path(),
        roots.as_slice(),
        excluded.as_slice(),
        selection.prune(),
        &mut left_out,
        &CopyCeiling::unbounded(),
    )
    .map_err(reason_of)?;
    // A copy's instant is in milliseconds since the epoch, and a change time in nanoseconds since it.
    let since: i128 = i128::try_from(u128::min(made_at_millis, u128::MAX / 1_000_000)).unwrap_or(i128::MAX / 1_000_000) * 1_000_000;
    let mut newer: Vec<Vec<u8>> = Vec::new();
    for entry in &entries {
        // A directory holds no bytes of its own: a change inside it is told by the files below it that changed, and a
        //  directory entry in this list would name a folder the person cannot compare with anything.
        if entry.kind == EntryKind::Directory {
            continue;
        }
        let Ok(path) = filesystem_path(source.root.as_path(), entry.relative_path.as_slice()) else { continue };
        // An entry the source removed since the walk has nothing to say, and a change time it cannot read proves nothing.
        let Ok(metadata) = std::fs::symlink_metadata(path.as_path()) else { continue };
        let Some(stamp) = stamp_of(&metadata) else { continue };
        if stamp_nanos(stamp.changed) > since {
            newer.push(entry.relative_path.clone());
        }
    }
    return Ok(newer);
}

/// What:
///  A length of time in the words a person reads,
/// whole seconds up to a minute and minutes above it.
/// Why:
///   The line says how old the copy is,
/// and "37 s" beside "2 minutes and 4 seconds" would be two voices for one fact;
/// a reader
///  who sees "less than a second" knows the file was written while the add ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ageWords = (seconds) => seconds === 0 ? 'less than a second' : seconds < 60 ? `${seconds} seconds` : `${minutes} minutes` + (rest ? ` and ${rest} seconds` : '');
/// ```
fn age_words(seconds: u64) -> String {
    if seconds == 0 {
        return String::from("less than a second");
    }
    if seconds < 60 {
        let noun: &str = if seconds == 1 { "second" } else { "seconds" };
        return format!("{seconds} {noun}");
    }
    let minutes: u64 = seconds / 60;
    let rest: u64 = seconds % 60;
    let minute_noun: &str = if minutes == 1 { "minute" } else { "minutes" };
    if rest == 0 {
        return format!("{minutes} {minute_noun}");
    }
    let rest_noun: &str = if rest == 1 { "second" } else { "seconds" };
    return format!("{minutes} {minute_noun} and {rest} {rest_noun}");
}

/// What:
///  The one line that says how old the ignored state of a new worktree is,
/// how many entries its copy is behind on,
/// and where every one of them is listed.
/// Why:
///   The person meets a worktree whose dependency folder is older than their own,
/// and nothing else says so;
/// the line names the source,
/// the age,
/// the first entries,
///  the count of the rest,
/// and the file that holds all of them,
/// so no one has to guess which files are older.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: the ignored state of ${destination} is a copy of ${source} that the wrapper made ${age} before this add, so ${count} ignored ${noun} changed after it: ${first}${more}. Every path is on its own line of ${file}. This worktree holds what each held then, or nothing of one written later; the source keeps every one of them, and the tool that writes them (a package install, a build) makes them again here.`
/// ```
fn notice_line(source_root: &Path, destination: &Path, age: &str, newer: &[Vec<u8>], list: Option<&Path>) -> String {
    let mut named: Vec<String> = Vec::new();
    for path in newer.iter().take(NOTICE_PATHS) {
        named.push(quote(shown_repository_path(path.as_slice()).as_str()));
    }
    let rest: usize = newer.len().saturating_sub(NOTICE_PATHS);
    let noun: &str = if newer.len() == 1 { "entry" } else { "entries" };
    let more: String = if rest == 0 { String::new() } else { format!(", and {rest} more") };
    let listed: String = match list {
        Some(path) => format!("Every path is on its own line of {}", quote(path.to_string_lossy().as_ref())),
        None => String::from("The paths here are every one this command found, because the file it lists them in could not be written"),
    };
    return format!(
        "monochromatic-git-policy-cli: the ignored state of {} is a copy of {} that the wrapper made {age} before this add, so {} ignored {noun} changed after it: {}{more}. {listed}. This worktree holds what each held then, or nothing of one written later; the source keeps every one of them, and the tool that writes them (a package install, a build) makes them again here.\n",
        quote(destination.to_string_lossy().as_ref()),
        quote(source_root.to_string_lossy().as_ref()),
        newer.len(),
        named.join(", ")
    );
}

/// What:
///  Say what the snapshot an add installed is behind on,
/// and list every path in the worktree's own Git directory.
/// Why:
///   A copy taken ahead of the add is the source as of an earlier instant,
/// and a person who is not told which entries
///  it is behind on cannot tell a stale dependency folder from a broken one.
/// The list stands in the worktree's administrative
///  directory,
/// where Git keeps what belongs to that worktree,
/// so it is never untracked and never in the way.
///  Nothing here fails the add:
/// the worktree is whole and Git made it,
/// and a line it cannot write is a line it does not print.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tellWhatTheCopyMisses({ context, source, destination, admin, madeAt, note }): void { const newer = entriesNewerThanTheCopy(...); if (newer.length === 0) return; const list = admin + '/' + LIST_NAME; writeEveryPath(list, newer); note.write(noticeLine(...)); }
/// ```
pub fn tell_what_the_copy_misses(context: &GitContext, source: &SnapshotSource, destination: &Path, admin: &Path, made_at_millis: u128, note: &mut dyn Write) {
    let newer: Vec<Vec<u8>> = match entries_newer_than_the_copy(context, source, destination, made_at_millis) {
        Ok(found) => found,
        Err(reason) => {
            crate::diagnostic_log::debug("worktreeIgnoredStateAfterTheCopy", format!("the ignored state the copy is behind on cannot be listed ({reason})").as_str());
            return;
        }
    };
    if newer.is_empty() {
        return;
    }
    let mut listing: String = String::new();
    for path in &newer {
        listing.push_str(quote(shown_repository_path(path.as_slice()).as_str()).as_str());
        listing.push('\n');
    }
    let list: PathBuf = admin.join(LIST_NAME);
    let written: bool = crate::worktree_copy_journal::write_atomic_unsynced(list.as_path(), listing.as_bytes(), "the entries the copy is behind on").is_ok();
    let now: u128 = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis();
    let age: String = age_words(u64::try_from(now.saturating_sub(made_at_millis) / 1000).unwrap_or(0));
    let line: String = notice_line(source.root.as_path(), destination, age.as_str(), newer.as_slice(), written.then_some(list.as_path()));
    let _written = note.write_all(line.as_bytes());
}

/// Controls of the copy's age stay out of the release executable.
#[cfg(test)]
#[path = "worktree_ignored_state_newer_than_the_copy_tests.rs"]
mod tests;
