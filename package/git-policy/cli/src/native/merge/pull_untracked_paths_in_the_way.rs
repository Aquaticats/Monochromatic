//! What:
//!  The files Git does not track that stand where a pull's move would write,
//!  copied beside the Git directory before the pull's failure is worded,
//!  and the sentence that says what was kept and where.
//! Why:
//!  When the fetch of a merging pull moves the checked-out branch,
//!  the wrapper moves the worktree along with `git read-tree -u -m`,
//!  as Git's `checkout_fast_forward` does.
//!  Git refuses that move when an untracked file stands at a path the new commit adds,
//!  and the failure keeps Git's own recovery words,
//!  whose `git diff <old head>` holds no untracked file,
//!  so a person who follows them loses the only copy of that file
//!  (the owner's rules "never discard user data" and
//!  "when a command must stop,
//! it names what it kept and where").
//!  The pull still has to stop as Git's does,
//!  so the wrapper copies those files first,
//! a plain file with its permission bits and a
//!  symbolic link as a link,
//! and names every copy,
//!  and it offers the worktree guard's hatch only where it kept everything the move would
//!  have overwritten.
//!  A path nothing could be copied from,
//! a directory that holds something standing where the
//!  new commit writes a file above all,
//! is named for the person to move out of the worktree first.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const inTheWay = keepTheUntrackedPathsInTheWay(context, '.', root, gitDir, from, to);
//! const words = inTheWay.listingFailure ?? recoveryWords(from, inTheWay);
//! ```

use crate::diagnostic_log::debug;
use crate::escape_hatch::WORKTREE_ENFORCEMENT_ESCAPE_HATCH;
use crate::kept_names_in_the_git_directories::data_folder_for_reading;
use crate::path_bytes::{os_from_path_bytes, path_bytes_of};
use crate::plain_file::open_plain_file;
use crate::shell_quote::shell_quote;
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use std::path::{Path, PathBuf};

/// Module tag for diagnostics.
const TAG: &str = "pullUntrackedPathsInTheWay";

/// The folder,
/// inside the Git directory's data folder,
/// that holds one numbered folder per
/// pull whose move could not run.
const KEPT_FOLDER_NAME: &str = "pull-kept";

/// How many numbered folders a keep is willing to look at.
/// A number no attempt found is reported instead,
/// so a folder tree nobody cleans costs one message and never a wait.
const KEPT_FOLDER_ATTEMPTS: u32 = 1000;

/// How many paths one sentence names before it says how many the folder holds besides.
/// Five is the bound the wrapper's other messages use for a list of paths.
const NAMED_PATHS_LIMIT: usize = 5;

/// What:
///  One file Git does not track,
/// where it stood in the worktree and where its copy is.
/// Why:
///   The person reads both in one sentence:
///  the path they know and the path they copy back from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type KeptPath = { inTheWorktree: string; inTheCopy: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeptPath {
    /// The path in the worktree,
    /// whose bytes the move would have overwritten.
    pub in_the_worktree: PathBuf,
    /// The copy of it,
    /// beside the Git directory.
    pub in_the_copy: PathBuf,
}

/// What:
///  One path the move would have overwritten that nothing was copied from,
///  and why.
/// Why:
///   The sentence offers the reset only after what stands there is moved away,
///  so the person has to know which paths those are and what stands there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PathLeftInTheWay = { inTheWorktree: string; why: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathLeftInTheWay {
    /// The path in the worktree.
    pub in_the_worktree: PathBuf,
    /// What stands there,
    /// or what stopped the copy.
    pub why: String,
}

/// What:
///  What the untracked files the move would have overwritten held,
///  and whether the wrapper could list them at all.
/// Why:
///   An empty answer is a promise that the reset discards nothing of the person's own;
///  a listing that could not run is no promise at all,
///  and the sentence says which it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UntrackedInTheWay = { listingFailure?: string; kept: KeptPath[]; left: PathLeftInTheWay[]; folder?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UntrackedInTheWay {
    /// Why the paths the move would overwrite could not be listed,
    /// if they could not.
    pub listing_failure: Option<String>,
    /// The files whose bytes are kept.
    pub kept: Vec<KeptPath>,
    /// The paths nothing was copied from.
    pub left: Vec<PathLeftInTheWay>,
    /// The folder the copies were made in,
    /// once one was made.
    pub folder: Option<PathBuf>,
}

/// What:
///  Whether one byte ends a name in Git's `-z` output.
/// Why:
///   A path may hold any other byte,
///  so the names are split on the NUL alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// byte === 0
/// ```
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  The words for a Git that could not be started,
///  which names the error the system gave and the one question that was asked.
/// Why:
///  The caller says what the listing was for,
/// a command that never started has no path of its own to name,
/// and one sentence here keeps every caller's words the same.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gitDiffTreeCouldNotStart(new Error('no such file or directory'));
/// // 'git diff-tree could not start: no such file or directory'
/// ```
fn git_diff_tree_could_not_start(error: std::io::Error) -> String {
    return format!("git diff-tree could not start: {error}");
}

/// What:
///  The paths the move from `from` to `to` adds,
///  as the bytes Git prints for them.
/// Why:
///   A path the new commit adds is the only kind of path at which a reset removes bytes Git
///  does not track:
///  a path both commits hold is tracked,
/// and its worktree copy is what `git diff` holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// exec('git', ['diff-tree', '-r', '--no-renames', '--name-only', '--diff-filter=A', '-z', from, to]);
/// ```
fn paths_the_move_adds(context: &GitContext, cwd: &Path, from: &str, to: &str) -> Result<Vec<Vec<u8>>, String> {
    let arguments: [&str; 8] = ["diff-tree", "-r", "--no-renames", "--name-only", "--diff-filter=A", "-z", from, to];
    let output: GitOutput = run_git(context, &GitRequest::new(cwd, &arguments)).map_err(git_diff_tree_could_not_start)?;
    if output.code != Some(0) {
        let status: String = match output.code {
            Some(code) => code.to_string(),
            None => String::from("none"),
        };
        return Err(format!("git diff-tree answered status {status}"));
    }
    let mut names: Vec<Vec<u8>> = Vec::new();
    for name in output.stdout.split(is_nul_byte) {
        if name.is_empty() {
            continue;
        }
        names.push(name.to_vec());
    }
    return Ok(names);
}

/// What:
///  A fresh numbered folder under the Git directory's data folder that this pull's copies go into.
/// Why:
///   A numbered folder is made with a create that fails when it exists,
///  so two pulls never write one folder,
///  and nothing this command writes replaces anything already there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let number = 1; number <= KEPT_FOLDER_ATTEMPTS; number += 1) { try { mkdir(`${parent}/${number}`); return `${parent}/${number}`; } catch (e) { if (e.code !== 'EEXIST') throw e; } }
/// ```
fn fresh_kept_folder(git_dir: &Path) -> std::io::Result<PathBuf> {
    let parent: PathBuf = data_folder_for_reading(git_dir).join(KEPT_FOLDER_NAME);
    std::fs::create_dir_all(&parent)?;
    for number in 1..=KEPT_FOLDER_ATTEMPTS {
        let candidate: PathBuf = parent.join(number.to_string());
        match std::fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "every numbered folder this command can make already exists"));
}

/// What:
///  Whether a directory holds no entry at all.
/// Why:
///   A directory standing where the new commit writes a file holds no bytes when it is empty,
///  so the message does not ask the person to move one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readdirSync(path).length === 0
/// ```
fn holds_nothing(directory: &Path) -> bool {
    return match std::fs::read_dir(directory) {
        Ok(mut listing) => listing.next().is_none(),
        Err(_unreadable) => false,
    };
}

/// What:
///  The folders that hold `in_the_copy`.
/// Why:
///   A copy keeps the path's own place under the folder,
///  so a path in a subdirectory needs its folders made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkdirSync(dirname(target), { recursive: true });
/// ```
fn make_the_copy_folder(in_the_copy: &Path) -> Result<(), String> {
    let Some(parent): Option<&Path> = in_the_copy.parent() else { return Ok(()) };
    if let Err(error) = std::fs::create_dir_all(parent) {
        return Err(format!("the folder {} of its copy could not be made: {error}", parent.display()));
    }
    return Ok(());
}

/// What:
///  A copy of one plain file,
///  opened the way every plain file of a worktree is opened:
///  without blocking and without following a link.
/// Why:
///   A name swapped for a named pipe between the look and the open would otherwise wait for a
///  writer that may never come,
///  and the pull holds nothing while it copies.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// copyFileSync(openPlain(source), target);
/// ```
fn copy_plain_file(from: &Path, to: &Path, metadata: &std::fs::Metadata) -> Result<(), String> {
    let mut source: std::fs::File = match open_plain_file(from) {
        Ok(opened) => opened,
        Err(error) => return Err(format!("the wrapper could not open {}: {error}", from.display())),
    };
    let mut destination: std::fs::File = match std::fs::File::create(to) {
        Ok(made) => made,
        Err(error) => return Err(format!("its copy at {} could not be made: {error}", to.display())),
    };
    if let Err(error) = std::io::copy(&mut source, &mut destination) {
        return Err(format!("its copy at {} could not be written: {error}", to.display()));
    }
    return copy_the_mode(metadata, to);
}

/// What:
///  Give the copy the permission bits the original holds.
/// Why:
///   An entry a person runs is its bytes and its mode,
///  so a copy that is not executable would not be the file they had.
#[cfg(unix)]
fn copy_the_mode(metadata: &std::fs::Metadata, to: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt as _;
    let mode: u32 = metadata.permissions().mode() & 0o777;
    if let Err(error) = std::fs::set_permissions(to, std::fs::Permissions::from_mode(mode)) {
        return Err(format!("the mode of its copy at {} could not be set: {error}", to.display()));
    }
    return Ok(());
}

/// The same on a platform whose permissions are not Unix bits:
///  the copy keeps what `std::fs::File::create` gave it.
#[cfg(not(unix))]
fn copy_the_mode(_metadata: &std::fs::Metadata, _to: &Path) -> Result<(), String> {
    return Ok(());
}

/// What:
///  Copy what stands at one path the move would overwrite into `in_the_copy`.
/// Why:
///   A plain file is copied byte for byte and a symbolic link is made again with its own target,
///  so the copy stands for the person's entry as the worktree held it;
///  anything else is answered with what stands there,
///  because a directory's contents and a named pipe are not one file this can keep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const metadata = lstatSync(path); if (metadata.isDirectory()) throw 'it is a directory'; if (metadata.isSymbolicLink()) symlinkSync(readlinkSync(path), target); else copyPlainFile(path, target);
/// ```
fn keep_one(in_the_worktree: &Path, in_the_copy: &Path) -> Result<(), String> {
    let metadata: std::fs::Metadata = match std::fs::symlink_metadata(in_the_worktree) {
        Ok(seen) => seen,
        Err(error) => return Err(format!("the wrapper could not read {}: {error}", in_the_worktree.display())),
    };
    let kind: std::fs::FileType = metadata.file_type();
    if kind.is_dir() {
        return Err(String::from("it is a directory, and the wrapper copies plain files and symbolic links"));
    }
    make_the_copy_folder(in_the_copy)?;
    if kind.is_symlink() {
        return copy_symbolic_link(in_the_worktree, in_the_copy);
    }
    if kind.is_file() {
        return copy_plain_file(in_the_worktree, in_the_copy, &metadata);
    }
    return Err(String::from("it is neither a plain file nor a symbolic link"));
}

/// What:
///  The link at `in_the_copy` that names what the link at `in_the_worktree` named.
/// Why:
///   A symbolic link's bytes are its target's text,
///  so the copy is made with that text and never by following it.
#[cfg(unix)]
fn copy_symbolic_link(in_the_worktree: &Path, in_the_copy: &Path) -> Result<(), String> {
    let target: PathBuf = match std::fs::read_link(in_the_worktree) {
        Ok(named) => named,
        Err(error) => return Err(format!("the wrapper could not read the link target of {}: {error}", in_the_worktree.display())),
    };
    if let Err(error) = std::os::unix::fs::symlink(target, in_the_copy) {
        return Err(format!("the link at {} could not be made: {error}", in_the_copy.display()));
    }
    return Ok(());
}

/// The same for a platform without Unix links,
///  which keeps the words and refuses the entry:
///  no gate builds this crate there.
#[cfg(not(unix))]
fn copy_symbolic_link(_in_the_worktree: &Path, _in_the_copy: &Path) -> Result<(), String> {
    return Err(String::from("it is a symbolic link, which this platform's copy cannot make"));
}

/// What:
///  Keep every file Git does not track that stands where the move from `from` to `to` writes.
/// Why:
///   The pull stops where Git's own does,
///  and the person keeps the bytes their file held,
///  named by the sentence `recovery_words` builds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const added = pathsTheMoveAdds(context, worktreeRoot, from, to); for (const name of added) { const path = join(root, name); ... }
/// ```
pub fn keep_the_untracked_paths_in_the_way(context: &GitContext, worktree_root: &Path, git_dir: &Path, from: &str, to: &str) -> UntrackedInTheWay {
    let added: Vec<Vec<u8>> = match paths_the_move_adds(context, worktree_root, from, to) {
        Ok(names) => names,
        Err(why) => {
            debug(TAG, format!("the paths the move would overwrite could not be listed: {why}").as_str());
            return UntrackedInTheWay { listing_failure: Some(why), kept: Vec::new(), left: Vec::new(), folder: None };
        }
    };
    let mut kept: Vec<KeptPath> = Vec::new();
    let mut left: Vec<PathLeftInTheWay> = Vec::new();
    let mut folder: Option<PathBuf> = None;
    for name in added {
        let in_the_worktree: PathBuf = worktree_root.join(os_from_path_bytes(name.as_slice()));
        match std::fs::symlink_metadata(&in_the_worktree) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                left.push(PathLeftInTheWay { in_the_worktree, why: format!("the wrapper could not read it: {error}") });
                continue;
            }
            // An empty directory holds no bytes to keep, and Git's reset removes it with nothing lost.
            Ok(stands) if stands.is_dir() && holds_nothing(in_the_worktree.as_path()) => continue,
            Ok(_stands) => {}
        }
        if folder.is_none() {
            match fresh_kept_folder(git_dir) {
                Ok(made) => folder = Some(made),
                Err(error) => {
                    left.push(PathLeftInTheWay { in_the_worktree, why: format!("the folder for its copy could not be made: {error}") });
                    continue;
                }
            }
        }
        let in_the_copy: PathBuf = match &folder {
            Some(made) => made.join(os_from_path_bytes(name.as_slice())),
            None => continue,
        };
        match keep_one(&in_the_worktree, &in_the_copy) {
            Ok(()) => kept.push(KeptPath { in_the_worktree, in_the_copy }),
            Err(why) => left.push(PathLeftInTheWay { in_the_worktree, why }),
        }
    }
    debug(
        TAG,
        format!("the move would have overwritten {} untracked path(s); {} kept, {} left for the person to move", kept.len() + left.len(), kept.len(), left.len()).as_str(),
    );
    return UntrackedInTheWay { listing_failure: None, kept, left, folder };
}

/// What:
///  A path as the shell reads it back,
///  between the backticks a message puts around a command word.
/// Why:
///   The sentence names paths a person copies back with their shell,
///  so a space or a quote in one is quoted rather than left to split the word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `\`${shellQuote(path)}\``
/// ```
fn quoted(path: &Path) -> String {
    return format!("`{}`", shell_quote(path_bytes_of(path)));
}

/// What:
///  The sentence that says where the copies of the kept files are.
/// Why:
///   The person has to reach the copies,
///  so each kept path is named with the copy beside the Git directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `The wrapper copied ${kept.map((one) => `${one.path} to ${one.copy}`).join(', ')} first`
/// ```
fn kept_sentence(kept: &[KeptPath], folder: Option<&Path>) -> String {
    let mut items: Vec<String> = Vec::new();
    for entry in kept.iter().take(NAMED_PATHS_LIMIT) {
        items.push(format!("{} to {}", quoted(entry.in_the_worktree.as_path()), quoted(entry.in_the_copy.as_path())));
    }
    let mut words: String = format!("The wrapper copied {} first", items.join(", "));
    if kept.len() > NAMED_PATHS_LIMIT {
        let beside: String = match folder {
            Some(made) => format!(" under {}", quoted(made)),
            None => String::new(),
        };
        words.push_str(format!(", and {} more{beside}", kept.len() - NAMED_PATHS_LIMIT).as_str());
    }
    return format!("{words}; copy them back from there when you want them, and delete those copies when you do not.\n");
}

/// What:
///  The sentence that names what nothing was copied from and says to move it first.
/// Why:
///   The reset deletes what stands there,
///  and the hatch is offered in this sentence and not before it,
///  so a reader who follows the words in order moves the path before the reset runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `nothing was copied from ${items}, and git diff ${orig} cannot hold it because Git does not track it; ...`
/// ```
fn left_sentence(left: &[PathLeftInTheWay], orig_head: &str) -> String {
    let mut items: Vec<String> = Vec::new();
    for entry in left.iter().take(NAMED_PATHS_LIMIT) {
        items.push(format!("{} ({})", quoted(entry.in_the_worktree.as_path()), entry.why));
    }
    let mut words: String = format!("monochromatic-git-policy-cli: nothing was copied from {}", items.join(", "));
    if left.len() > NAMED_PATHS_LIMIT {
        words.push_str(format!(" and {} more", left.len() - NAMED_PATHS_LIMIT).as_str());
    }
    words.push_str(format!(", and `git diff {orig_head}` cannot hold what stands there because Git does not track it; the reset deletes it. Move what stands there out of the worktree first, then run `git reset {WORKTREE_ENFORCEMENT_ESCAPE_HATCH} --hard` if you mean it.\n").as_str());
    return words;
}

/// What:
///  The sentence for a listing that could not run,
///  which keeps nothing and offers the reset only after the untracked files are saved.
/// Why:
///   The wrapper cannot say the reset keeps nothing of the person's,
///  so it says what it does not know and what to do about it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the files this move would overwrite could not be listed (${why}), so nothing could be kept. Save what Git does not track (git ls-files --others) before ...`
/// ```
fn unlisted_sentence(why: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: the files this move would overwrite could not be listed ({why}), so nothing could be kept; save what Git does not track (`git ls-files --others`) before you run `git reset {WORKTREE_ENFORCEMENT_ESCAPE_HATCH} --hard`.\n"
    );
}

/// What:
///  The whole failure a pull prints when its move could not run:
///  Git's own recovery words,
///  the wrapper's line about the worktree guard,
///  and what it kept.
/// Why:
///   Git's words stay as Git prints them (`SPEC.md`,
///  "Pull");
///  the wrapper adds what Git cannot say,
///  that its guard refuses the reset,
///  and that the untracked files Git's diff omits are kept.
///  The hatch appears exactly once:
///  in the sentence that offers the reset,
///  which is the guard's own sentence only when nothing the move would overwrite was left
///  uncopied.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gitsRecoveryWords(orig) + guardSentence(inTheWay) + (kept.length ? keptSentence(kept) : '') + (left.length ? leftSentence(left) : '') + (listingFailure ? unlistedSentence(listingFailure) : '');
/// ```
pub fn recovery_words(orig_head: &str, in_the_way: &UntrackedInTheWay) -> String {
    let mut words: String = format!(
        "fatal: Cannot fast-forward your working tree.\nAfter making sure that you saved anything precious from\n$ git diff {orig_head}\noutput, run\n$ git reset --hard\nto recover.\nmonochromatic-git-policy-cli: that reset discards whatever this worktree holds, and the wrapper's worktree guard refuses a hard reset here without {WORKTREE_ENFORCEMENT_ESCAPE_HATCH}, which is what made this fast-forward fail"
    );
    let offering: bool = in_the_way.listing_failure.is_none() && in_the_way.left.is_empty();
    if offering {
        words.push_str(format!("; run `git reset {WORKTREE_ENFORCEMENT_ESCAPE_HATCH} --hard` if you mean it.\n").as_str());
    } else {
        words.push_str(".\n");
    }
    if !in_the_way.kept.is_empty() {
        words.push_str("monochromatic-git-policy-cli: the move would have overwritten files Git does not track, which `git diff ");
        words.push_str(orig_head);
        words.push_str("` does not hold. ");
        words.push_str(kept_sentence(in_the_way.kept.as_slice(), in_the_way.folder.as_deref()).as_str());
    }
    if !in_the_way.left.is_empty() {
        words.push_str(left_sentence(in_the_way.left.as_slice(), orig_head).as_str());
    }
    if let Some(why) = &in_the_way.listing_failure {
        words.push_str(unlisted_sentence(why).as_str());
    }
    return words;
}

/// Controls for the paths a pull keeps and the words it prints stay out of the release executable.
#[cfg(test)]
#[path = "pull_untracked_paths_in_the_way_tests.rs"]
mod tests;
