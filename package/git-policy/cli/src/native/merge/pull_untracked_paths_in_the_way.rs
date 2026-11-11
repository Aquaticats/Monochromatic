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
//! a file or a link standing on the way to an added path included,
//!  because the move needs a folder there and the reset replaces it,
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
use crate::path_bytes::{os_from_path_bytes, path_bytes_of};
use crate::pull_untracked_paths_in_the_way_kept_copies::{fresh_kept_folder, holds_nothing, keep_one};
use crate::shell_quote::shell_quote;
use crate::repository_selection::of_paths;
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Module tag for diagnostics.
const TAG: &str = "pullUntrackedPathsInTheWay";

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
///  The paths the move from `from` to `to` takes:
/// the ones it adds,
///  and the ones the old commit tracked and it stops tracking.
/// Why:
///   An added path is where the reset removes bytes Git does not track;
///  a path the old commit tracked and the move takes away
///  is where the same reset removes bytes `git diff <old head>`
///  holds,
///  so the two lists together say which entry standing there is the person's to keep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PathsTheMoveTakes = { added: Uint8Array[]; gone: Set<Uint8Array> };
/// ```
struct PathsTheMoveTakes {
    /// The paths the new commit adds,
    /// as the bytes Git prints for them.
    added: Vec<Vec<u8>>,
    /// The paths the old commit tracked and the new one does not.
    gone: HashSet<Vec<u8>>,
}

/// What:
///  The paths the move from `from` to `to` adds,
///  and those it stops tracking,
///  as the bytes Git prints for them.
/// Why:
///   The request runs from the top of the worktree,
///  which is a directory Git answered,
///  so it keeps none of the caller's global options:
///  a relative `-C` or a relative `--git-dir` there is read from the directory Git starts in,
///  and starting Git anywhere but the caller's own directory applies it a second time
///  and names a repository that does not exist
///  (`repository_selection`,
///  "The repository a global prefix selects").
///  The repository itself is named by its absolute paths instead,
///  so the query reads the one the caller selected
///  from a directory the wrapper already resolved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// request.inTheCallersRepository(repositorySelectionOfPaths(gitDir, root))
/// const request = gitRequest(root, ['diff-tree', '-r', '--no-renames', '--name-status', '--diff-filter=AD', '-z', from, to]).withoutPrefix();
/// ```
fn paths_the_move_takes(context: &GitContext, worktree_root: &Path, git_dir: &Path, from: &str, to: &str) -> Result<PathsTheMoveTakes, String> {
    let arguments: [&str; 8] = ["diff-tree", "-r", "--no-renames", "--name-status", "--diff-filter=AD", "-z", from, to];
    let selection: Vec<std::ffi::OsString> = of_paths(git_dir, worktree_root);
    let mut request: GitRequest = GitRequest::new(worktree_root, &arguments).in_the_callers_repository(selection);
    request.without_prefix = true;
    let output: GitOutput = run_git(context, &request).map_err(git_diff_tree_could_not_start)?;
    if output.code != Some(0) {
        let status: String = match output.code {
            Some(code) => code.to_string(),
            None => String::from("none"),
        };
        return Err(format!("git diff-tree answered status {status}"));
    }
    let mut takes: PathsTheMoveTakes = PathsTheMoveTakes { added: Vec::new(), gone: HashSet::new() };
    // Each entry of `--name-status -z` is the status byte, a NUL, the path, and a NUL.
    let fields: Vec<&[u8]> = output.stdout.split(is_nul_byte).collect();
    for pair in fields.chunks(2) {
        let (Some(status), Some(name)) = (pair.first(), pair.get(1)) else { break };
        let code: u8 = match status.first() {
            Some(byte) => *byte,
            None => break,
        };
        match code {
            b'A' => takes.added.push(name.to_vec()),
            b'D' => {
                takes.gone.insert(name.to_vec());
            }
            other => return Err(format!("git diff-tree answered a status this does not read: {}", char::from(other))),
        }
    }
    return Ok(takes);
}

/// What:
///  Whether one byte separates two components of a path Git listed.
/// Why:
///   A tree writes every path with `/` on every platform,
///  and its components are never empty,
///  so a name splits into exactly its folders and its own last component.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// byte === 0x2f
/// ```
fn is_path_separator(byte: &u8) -> bool {
    return *byte == b'/';
}

/// What:
///  The first entry along one added path that the move would remove,
///  or nothing when nothing Git does not track stands there.
/// Why:
///   The move needs a folder at every component above the path it writes,
///  and the reset Git's words offer deletes an untracked file or link standing at one of them,
///  so such an entry is the person's to keep exactly as the added path itself is.
///  A component the old commit tracked as a file is replaced on purpose,
///  and its bytes are in `git diff <old head>`,
///  so nothing is kept for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StandingInTheWay = { kind: 'nothing' } | { kind: 'copy'; below: string } | { kind: 'name'; below: string; why: string };
/// ```
enum StandingInTheWay {
    /// Nothing Git does not track stands where the move writes.
    Nothing,
    /// An entry the move would remove,
    /// by its path below the worktree root;
    /// `keep_one` copies it or says why it cannot.
    Copy(PathBuf),
    /// A path the wrapper cannot copy,
    /// by its path below the worktree root,
    /// and why.
    Name(PathBuf, String),
}

/// What:
///  The first entry along `name` the move from `from` to `to` would remove.
/// Why:
///   A folder on the way is walked through,
///  an empty folder where the new commit writes a file holds no bytes,
///  and everything else is the entry the person's bytes stand in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function standingInTheWay(root, name, gone) { let below = ''; const parts = name.split('/'); for (const [at, part] of parts.entries()) { below = below ? `${below}/${part}` : part; if (at + 1 < parts.length && gone.has(below)) return { kind: 'nothing' }; const seen = lstat(join(root, below)); if (missing(seen)) return { kind: 'nothing' }; if (seen.isDirectory() && (at + 1 < parts.length || holdsNothing(join(root, below)))) continue; return { kind: 'copy', below }; } return { kind: 'nothing' }; }
/// ```
fn standing_in_the_way(worktree_root: &Path, name: &[u8], gone: &HashSet<Vec<u8>>) -> StandingInTheWay {
    let components: Vec<&[u8]> = name.split(is_path_separator).collect();
    let mut below: PathBuf = PathBuf::new();
    for (position, component) in components.iter().enumerate() {
        below.push(os_from_path_bytes(component));
        let is_the_path_itself: bool = position + 1 == components.len();
        if !is_the_path_itself && gone.contains(path_bytes_of(below.as_path())) {
            return StandingInTheWay::Nothing;
        }
        let in_the_worktree: PathBuf = worktree_root.join(below.as_path());
        let stands: std::fs::Metadata = match std::fs::symlink_metadata(in_the_worktree.as_path()) {
            Ok(seen) => seen,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return StandingInTheWay::Nothing,
            Err(error) => return StandingInTheWay::Name(below, format!("the wrapper could not read it: {error}")),
        };
        if stands.is_dir() && (!is_the_path_itself || holds_nothing(in_the_worktree.as_path())) {
            continue;
        }
        return StandingInTheWay::Copy(below);
    }
    return StandingInTheWay::Nothing;
}

/// What:
///  Whether the list of paths nothing was copied from already names this path.
/// Why:
///   Two added paths below one entry both walk to it,
///  and the person reads one line for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// left.some((one) => one.path === path)
/// ```
fn names_the_path(left: &[PathLeftInTheWay], in_the_worktree: &Path) -> bool {
    for one in left {
        if one.in_the_worktree == in_the_worktree {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether the list of copies already holds this path.
/// Why:
///   Two added paths below one entry both walk to it,
///  and one copy of it is what the person restores.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// kept.some((one) => one.path === path)
/// ```
fn keeps_the_path(kept: &[KeptPath], in_the_worktree: &Path) -> bool {
    for one in kept {
        if one.in_the_worktree == in_the_worktree {
            return true;
        }
    }
    return false;
}

/// What:
///  One path nothing was copied from,
///  named once however many added paths stand behind it.
/// Why:
///   Two added paths below one entry both walk to it,
///  and the person reads one line for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!left.some((one) => one.path === path)) left.push({ path, why });
/// ```
fn name_the_path_once(left: &mut Vec<PathLeftInTheWay>, in_the_worktree: PathBuf, why: String) {
    if names_the_path(left.as_slice(), in_the_worktree.as_path()) {
        return;
    }
    left.push(PathLeftInTheWay { in_the_worktree, why });
}


/// What:
///  Keep every file Git does not track that stands where the move from `from` to `to` writes,
///  or on the way to it.
/// Why:
///   The pull stops where Git's own does,
///  and the person keeps the bytes their file held,
///  named by the sentence `recovery_words` builds.
///  A file or a link standing where the move needs a folder is kept the same way,
///  because the reset Git's words offer replaces it with that folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const takes = pathsTheMoveTakes(context, worktreeRoot, gitDir, from, to); for (const name of takes.added) { const below = standingInTheWay(worktreeRoot, name, takes.gone); ... }
/// ```
pub fn keep_the_untracked_paths_in_the_way(context: &GitContext, worktree_root: &Path, git_dir: &Path, from: &str, to: &str) -> UntrackedInTheWay {
    let takes: PathsTheMoveTakes = match paths_the_move_takes(context, worktree_root, git_dir, from, to) {
        Ok(taken) => taken,
        Err(why) => {
            debug(TAG, format!("the paths the move would overwrite could not be listed: {why}").as_str());
            return UntrackedInTheWay { listing_failure: Some(why), kept: Vec::new(), left: Vec::new(), folder: None };
        }
    };
    let mut kept: Vec<KeptPath> = Vec::new();
    let mut left: Vec<PathLeftInTheWay> = Vec::new();
    let mut folder: Option<PathBuf> = None;
    for name in takes.added {
        let below: PathBuf = match standing_in_the_way(worktree_root, name.as_slice(), &takes.gone) {
            StandingInTheWay::Nothing => continue,
            StandingInTheWay::Name(below, why) => {
                name_the_path_once(&mut left, worktree_root.join(below.as_path()), why);
                continue;
            }
            StandingInTheWay::Copy(below) => below,
        };
        let in_the_worktree: PathBuf = worktree_root.join(below.as_path());
        if keeps_the_path(kept.as_slice(), in_the_worktree.as_path()) || names_the_path(left.as_slice(), in_the_worktree.as_path()) {
            continue;
        }
        if folder.is_none() {
            match fresh_kept_folder(git_dir) {
                Ok(made) => folder = Some(made),
                Err(error) => {
                    name_the_path_once(&mut left, in_the_worktree, format!("the folder for its copy could not be made: {error}"));
                    continue;
                }
            }
        }
        let in_the_copy: PathBuf = match &folder {
            Some(made) => made.join(below.as_path()),
            None => continue,
        };
        match keep_one(&in_the_worktree, &in_the_copy) {
            Ok(()) => kept.push(KeptPath { in_the_worktree, in_the_copy }),
            Err(why) => name_the_path_once(&mut left, in_the_worktree, why),
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
