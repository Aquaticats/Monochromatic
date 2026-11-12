//! What:
//!  The entries of the person's that stand at an ancestor of a path a merge writes:
//!  finding them,
//!  keeping a copy of each beside it before
//!  Git's update needs a directory there,
//!  deciding the copies a stopped command left,
//!  and the line that says where each one is.
//! Why:
//!  Creating the directory an added path needs removes whatever entry stands on the way to it:
//! measured,
//!  an ignored regular file `dir` was gone after a merge that added `dir/x.txt`,
//!  and so was an ignored symbolic
//!  link `dir` to a directory,
//!  with no copy and no notice,
//!  while a path the merge writes itself is carried.
//!  A directory and the person's file cannot share one path,
//!  so the merge keeps its own content there and the person's entry
//!  is kept whole beside it,
//!  as a take keeps the file a merge replaces (`SPEC.md`,
//! "the copy a merge keeps beside a path").
//!  The entry is copied rather than moved:
//! the copy is written before Git's own update and the entry itself stays where it is
//!  until that update removes it,
//!  so a merge that stops before the update leaves the checkout exactly as it was,
//!  and a copy whose
//!  bytes the entry at the path still holds is removed by the next command as a duplicate.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const entries = theEntriesADirectoryWouldReplace(root, written);
//! const kept = keepThemAside(context, root, gitDir, entries, transactionId);
//! stderr.write(theLine(reference, kept));
//! ```

/// The paths the merge writes,
/// spelled as worktree paths.
use crate::merge_isolation_changes::worktree_path;
/// The bytes of one entry,
/// which a directory,
/// a pipe,
/// a socket or a device has none of.
use crate::merge_isolation_carry::bytes_at;
/// The one step that writes one entry's bytes or its link target under a name beside it.
use crate::merge_isolation_carry_place::write_entry;
/// The record that names a copy beside a path,
/// so a later command decides it again,
/// and the decision every command makes about such a copy.
use crate::merge_isolation_kept_copies::{KeptCopy, keep_the_copy};
use crate::merge_isolation_kept_copies_decisions::settle_kept_copy_awaiting;
/// The private name a copy is written under,
/// every name beside one path,
/// and the keys of the merges that are still running.
use crate::merge_isolation_take_aside_names::{aside_name, asides_of, live_merge_keys};
/// The entry of the process runtime a synchronous caller takes.
use crate::lazy_runtime::run_on_runtime;
/// Real Git.
use crate::transaction_git::GitContext;
/// Writing the line a person reads.
use std::io::Write;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// How many times one entry's bytes are read and copied before the newest copy is kept.
/// Why three:
///  a save that arrives between the read and the copy of one attempt is copied by the next,
///  and a file a program
///  writes faster than twice in a row is not one this command can follow;
/// the copy of the last attempt holds the bytes the merge
///  then removed.
const COPY_ATTEMPTS: usize = 3;

/// One entry of the person's that a copy was made of,
/// and where the copy waits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type KeptEntry = { path: Uint8Array; aside: string };
/// ```
pub struct KeptEntry {
    /// The worktree path the entry stood at,
    /// from the top of the worktree.
    pub path: Vec<u8>,
    /// Where the copy of it waits.
    pub aside: PathBuf,
}

/// What:
///  The entries that stand at an ancestor of a path the merge writes,
///  when what stands there is not a directory,
///  and the merge does not write that path itself.
/// Why:
///   A merge that writes `dir/x.txt` needs a directory at `dir`,
///  and Git's own update removes whatever entry stands
///  there to make it;
///  a path the merge writes itself is the carry's own business,
///  and a directory is what the update needs
///  anyway,
///  so neither is asked about here.
///  Every path is read without following a link,
///  because a link to a directory is still an
///  entry Git removes and a link is a file whose own text is what a copy holds.
///  Nothing is opened:
///  a named pipe
///  is named by its type alone,
///  so a merge never waits for a writer that may never come.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theEntriesADirectoryWouldReplace(root, written) { return written.flatMap(ancestors).filter((path) => { const found = symlinkMetadataSync(join(root, path)); return found && !found.isDirectory(); }); }
/// ```
pub fn the_entries_a_directory_would_replace(root: &Path, written: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let mut found: Vec<Vec<u8>> = Vec::new();
    for path in written {
        for (index, byte) in path.iter().enumerate() {
            if *byte != b'/' || index == 0 {
                continue;
            }
            let ancestor: Vec<u8> = path[..index].to_vec();
            if written.contains(&ancestor) || found.contains(&ancestor) {
                continue;
            }
            let location: PathBuf = worktree_path(root, ancestor.as_slice());
            let Ok(metadata) = std::fs::symlink_metadata(location.as_path()) else { continue };
            if metadata.is_dir() {
                continue;
            }
            found.push(ancestor);
        }
    }
    found.sort();
    return found;
}

/// What:
///  Keep a copy of every entry beside it,
///  recorded so a later command decides it again.
/// Why:
///   The copy must exist before Git's own update removes the entry,
///  and the record must exist before the copy does,
///  so a
///  command killed between the two leaves a copy the next command can account for.
///  An entry whose bytes cannot be read is left out:
///  its kind is
///  what the refusal about a named pipe,
///  a socket or a device names,
///  and that refusal runs before this step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keepThemAside(root, gitDir, entries, transactionId): KeptEntry[];
/// ```
pub fn keep_them_aside(root: &Path, git_dir: &Path, entries: &[Vec<u8>], transaction_id: &str) -> Result<Vec<KeptEntry>, String> {
    let mut kept: Vec<KeptEntry> = Vec::new();
    for path in entries {
        match copy_one(root, git_dir, path, transaction_id) {
            Ok(Some(entry)) => kept.push(entry),
            Ok(None) => {}
            Err(reason) => return Err(reason),
        }
    }
    return Ok(kept);
}

/// What:
///  One entry's copy,
///  written again when the entry changed while the first copy was made.
/// Why:
///  A save between the read and the copy is the person's newest bytes,
///  and the merge's own update removes the entry
///  with the same window between its judgement and its write;
///  copying again narrows it to the width of one read,
///  and the
///  last copy holds the bytes the update took.
///  A read that finds nothing left means the update already removed the entry,
///  and the copy of the
///  attempt before it is the one to keep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copyOne(root, gitDir, path, transactionId): KeptEntry | undefined;
/// ```
fn copy_one(root: &Path, git_dir: &Path, path: &[u8], transaction_id: &str) -> Result<Option<KeptEntry>, String> {
    let location: PathBuf = worktree_path(root, path);
    let mut kept: Option<KeptEntry> = None;
    for _attempt in 0..COPY_ATTEMPTS {
        let Some((bytes, mode)) = bytes_at(location.as_path())? else { return Ok(kept) };
        let aside: PathBuf = aside_name(location.as_path(), transaction_id);
        // A copy that cannot be written is a name of its own to remove, so nothing waits beside the path.
        if let Err(reason) = write_entry(aside.as_path(), bytes.as_slice(), mode) {
            let _left = std::fs::remove_file(aside.as_path());
            return Err(format!("{} could not be kept beside itself: {reason}", location.display()));
        }
        keep_the_copy(git_dir, location.as_path(), aside.as_path());
        if let Some(older) = kept.take() {
            // The bytes of the older copy are older than the ones just written, which a later command removes as a
            //  duplicate of nothing; its record goes with it when the sweep finds the name gone.
            let _dropped = std::fs::remove_file(older.aside.as_path());
        }
        kept = Some(KeptEntry { path: path.to_vec(), aside });
        match bytes_at(location.as_path())? {
            // The entry still holds the bytes the copy does, so the copy is the entry that Git's update takes.
            Some((now, now_mode)) if now == bytes && now_mode == mode => return Ok(kept),
            // The entry is gone: the update took it, and the copy is the only one left.
            None => return Ok(kept),
            // The person wrote it again, so the next attempt copies the newer bytes.
            Some(_newer) => {}
        }
    }
    return Ok(kept);
}

/// What:
///  Decide the copies that wait beside these entries,
///  which a merge that stopped left.
/// Why:
///   A merge killed before Git's own update leaves a copy beside an entry that still stands where it was,
///  and a merge
///  killed after the update leaves only the copy;
///  one decision serves both:
///  the copy goes back when the path is free and the merge that
///  made it is unfinished,
///  it is removed when the entry at the path holds its bytes,
///  and it stays and is named when
///  nothing proves either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function settleTheCopiesThere(context, root, gitDir, entries, stderr) { for (const aside of asidesOf(location)) settleKeptCopy({ ..., change: undefined }); }
/// ```
pub fn settle_the_copies_there(context: &GitContext, root: &Path, git_dir: &Path, entries: &[Vec<u8>], out: &mut dyn Write) {
    let live: Vec<String> = live_merge_keys(git_dir);
    for path in entries {
        let location: PathBuf = worktree_path(root, path.as_slice());
        for aside in asides_of(location.as_path()) {
            let copy: KeptCopy = KeptCopy { location: location.clone(), aside: aside.path.clone() };
            // An entry that a merge puts a directory over has no change of its own, so no commit holds its bytes.
            let _put_back = run_on_runtime(settle_kept_copy_awaiting(context, root, git_dir, &copy, Some(&aside), None, None, live.as_slice(), out));
        }
    }
}

/// What:
///  The lines that say which entry was kept whole,
///  where its copy is,
///  and that the merge was made.
/// Why:
///   The person has to find their bytes to judge them,
///  and no command puts them back by itself while a
///  directory stands at the path,
///  so the line names the copy and says that the two commands that end it are theirs to run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// kept.map((one) => `${aside} holds ${path}, which this merge put a directory over; ...`);
/// ```
pub fn the_line(reference: &str, kept: &[KeptEntry]) -> String {
    let mut lines: String = String::new();
    for entry in kept {
        let path: String = crate::path_bytes::path_message_text(entry.path.as_slice());
        let aside: String = crate::shell_quote::shell_quote(entry.aside.as_os_str().as_encoded_bytes());
        lines.push_str(
            format!(
                "monochromatic-git-policy-cli: git merge {reference}: this merge writes a directory at {path}, and the entry that \
                 stood there is kept whole at {}, which is where {} stood. The merge was made and nothing of yours was removed. \
                 `mv -- {aside} <a name of your own>` keeps it under that name, and `rm -- {aside}` drops it; a directory stands at \
                 {path}, so no later git command puts the copy back for you.\n",
                entry.aside.display(),
                path
            )
            .as_str(),
        );
    }
    return lines;
}
