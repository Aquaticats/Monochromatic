//! What:
//!  The questions a repair asks of a checkout's files, and the one write that puts the branch's own
//!  entries back over the paths a move left holding the old tip's.
//! Why:
//!  What a checkout holds is three answers, not one: the branch it shows, the index against that
//!  branch, and every path of the move in the worktree.
//!  Each question here is one Git command, so the repair that decides what to write reads them in
//!  one place and the file that words the answer does not grow a second copy of them.
//!  These questions are kept apart from `auto_push_reconcile_branch_checkout.rs` because that file
//!  reached the repository's line budget, and because a path whose content is proven is a different
//!  subject from the words the report prints.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const disagreeing = await git(['diff','--name-only','-z','--no-renames','HEAD','--',...paths]);
//! const stale = paths.filter((path) => !(await git(['diff','--name-only','-z','--no-renames',local,'--',path])));
//! await git(['checkout','--quiet','HEAD','--',...stale]);
//! ```

use crate::auto_push_dirty_paths::diff_paths;
use crate::auto_push_reconcile_basics_and_limits::argv;
use crate::auto_push_single_flight::{PushGit, SingleFlightError, run_query_awaiting};
use std::ffi::OsString;

/// What:
///  The branch the worktree shows,
/// or nothing for a detached `HEAD`.
/// Why:
///   The report names it,
/// and the repair writes the tree of the branch it is,
/// which is not the branch the move was for once a switch took the checkout elsewhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shows = await git(['symbolic-ref','--quiet','HEAD']); // undefined for a detached HEAD
/// ```
pub(crate) async fn shows_awaiting(git: &PushGit) -> Result<Option<String>, SingleFlightError> {
    let asked = run_query_awaiting(git, &argv(&["symbolic-ref", "--quiet", "HEAD"])).await?;
    // What: `symbolic-ref --quiet` answers 1 for a detached HEAD, and 128 for a read that failed.
    // Why:  taking every failure for "detached" turned a failed read into an answer, and the repair then took
    // the direction for another branch and classed this branch's own moved-from entries as a person's, so the
    // record went and a commit recorded the reversal
    //  (a review of commit `7aef6368e7`, its finding 2).
    match asked.code {
        Some(0) => {}
        Some(1) => return Ok(None),
        other => {
            return Err(SingleFlightError::Git {
                arguments: String::from("symbolic-ref --quiet HEAD"),
                code: other,
                stderr: String::from_utf8_lossy(&asked.stderr).into_owned(),
            });
        }
    }
    let name: String = String::from_utf8_lossy(&asked.stdout).trim().to_owned();
    return Ok(if name.is_empty() { None } else { Some(name) });
}

/// What:
///  The commit this worktree's HEAD names,
/// asked again at every attempt of a repair.
/// Why:
///   A `git switch` can finish between any two of this command's questions, so the tree a checkout must hold
/// is the one its own HEAD names now, never one read earlier and cached
///  (a review of commit `7aef6368e7`, its findings 1 and 3).
///  A failure here is a read that failed and not an answer: the caller keeps the record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const head = await git(['rev-parse','--verify','HEAD^{commit}']); // status 0, or a read that failed
/// ```
pub(crate) async fn the_head_commit_awaiting(git: &PushGit) -> Result<String, SingleFlightError> {
    let asked = run_query_awaiting(git, &argv(&["rev-parse", "--verify", "HEAD^{commit}"])).await?;
    if asked.code != Some(0) {
        return Err(SingleFlightError::Git {
            arguments: String::from("rev-parse --verify HEAD^{commit}"),
            code: asked.code,
            stderr: String::from_utf8_lossy(&asked.stderr).into_owned(),
        });
    }
    return Ok(String::from_utf8_lossy(&asked.stdout).trim().to_owned());
}

/// What:
///  The Git arguments naming `paths`,
/// each as the operating system names it.
/// Why:
///   Every question below passes the paths of the move to Git as arguments,
/// so the bytes of a name are carried through one loop rather than a closure of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const path of paths) arguments.push(osFromPathBytes(path));
/// ```
fn naming(paths: &[Vec<u8>], mut arguments: Vec<OsString>) -> Vec<OsString> {
    for path in paths {
        arguments.push(crate::path_bytes::os_from_path_bytes(path.as_slice()));
    }
    return arguments;
}

/// What:
///  The paths of the move where the checkout does not agree with the branch it shows,
/// asked once so no place decides that on part of the answer.
/// Why:
///   `git diff <branch>` compares the index and the files to that branch in one answer,
/// so a reset that moves the index and leaves the files is a difference here,
///  while a place that asked only the index read the same state as agreement
///  (a review of commit `cae78e1d7e`, its finding 3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// diffPaths((await git(['--no-optional-locks','diff','--name-only','-z','--no-renames','HEAD','--',...paths])).stdout)
/// ```
pub(crate) async fn the_paths_the_checkout_disagrees_with_its_branch_awaiting(
    git: &PushGit,
    paths: &[Vec<u8>],
) -> Result<Vec<Vec<u8>>, SingleFlightError> {
    let arguments: Vec<OsString> =
        naming(paths, argv(&["--no-optional-locks", "diff", "--name-only", "-z", "--no-renames", "HEAD", "--"]));
    let listed = run_query_awaiting(git, &arguments).await?;
    if listed.code != Some(0) {
        return Err(SingleFlightError::Git {
            arguments: String::from("diff --name-only -z --no-renames HEAD"),
            code: listed.code,
            stderr: String::from_utf8_lossy(&listed.stderr).into_owned(),
        });
    }
    return Ok(diff_paths(&listed.stdout));
}

/// What:
///  The paths of `paths` where the entry `tree` holds is an entry this checkout still holds, in its index or in
/// its file.
/// Why:
///  What proves a path is a move's own leftover is an entry the move wrote there, and an absent file is not an
/// entry: `git diff <tree> -- <path>` answers "no difference" when neither the tree nor the worktree has the
/// path, so a file the merge added and a person then deleted read as the move's debris and the repair wrote it
/// back  (a review of commit `4a2425a378`, its finding 2), and `git rm --cached` with the file still there
/// stopped every command (its finding 3).
///  An absent entry in the index proves the same thing only in the checkout the move is for: there the absence is
/// the step the move has not written yet, while in a worktree that shows another branch an absent index entry is
/// what a person's `git rm` leaves, and writing it back would undo their removal (a review of commit
/// `4a2425a378`, its finding 2, its `git rm` case).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const present = await lsTree(tree, paths); return present.filter((path) => indexHoldsOrFileHolds(tree, path))
/// ```
pub(crate) async fn the_paths_the_move_wrote_awaiting(
    git: &PushGit,
    tree: &str,
    paths: &[Vec<u8>],
    an_absent_entry_counts: bool,
) -> Result<Vec<Vec<u8>>, SingleFlightError> {
    // What: a path an end of the move really wrote: the index holds that end's entry, or the file is there and
    // holds its bytes; an entry the tree does not have counts only in the checkout the move is for.
    // Why:  an absent file is not an entry: a file the merge added and a person then deleted, or untracked with
    // `git rm --cached`, holds nothing the move wrote, while a file still holding the moved-from bytes is the
    // move's own leftover and is brought forward
    //  (a review of commit `4a2425a378`, its findings 2 and 3; commit `cae78e1d7e`, its finding 3).
    let in_the_index: Vec<Vec<u8>> = the_paths_whose_index_holds_awaiting(git, tree, paths).await?;
    let in_the_file: Vec<Vec<u8>> = the_paths_still_holding_the_old_tip_awaiting(git, tree, paths).await?;
    let with_an_entry: Option<Vec<Vec<u8>>> = match an_absent_entry_counts {
        true => None,
        false => Some(the_paths_with_an_entry_in_the_tree_awaiting(git, tree, paths).await?),
    };
    let mut wrote: Vec<Vec<u8>> = Vec::new();
    for path in paths {
        let entry_is_there: bool = match &with_an_entry {
            Some(there) => there.contains(path),
            None => true,
        };
        if in_the_index.contains(path) && entry_is_there {
            wrote.push(path.clone());
            continue;
        }
        let present: bool = std::fs::symlink_metadata(git.cwd.join(crate::path_bytes::os_from_path_bytes(path.as_slice()))).is_ok();
        if present && in_the_file.contains(path) {
            wrote.push(path.clone());
        }
    }
    return Ok(wrote);
}

/// What:
///  The paths of `paths` that hold an entry in `tree` at all.
/// Why:
///  A path the tree has no entry for is what a `git rm` of a file the move added leaves behind, and the index's
/// silence there is not the tree's own state
///  (a review of commit `4a2425a378`, its finding 2, its `git rm` case).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// diffPaths((await git(['ls-tree','--name-only','-z',tree,'--',...paths])).stdout)
/// ```
pub(crate) async fn the_paths_with_an_entry_in_the_tree_awaiting(
    git: &PushGit,
    tree: &str,
    paths: &[Vec<u8>],
) -> Result<Vec<Vec<u8>>, SingleFlightError> {
    let arguments: Vec<OsString> = naming(paths, argv(&["--no-optional-locks", "ls-tree", "--name-only", "-z", tree, "--"]));
    let listed = run_query_awaiting(git, &arguments).await?;
    if listed.code != Some(0) {
        return Err(SingleFlightError::Git {
            arguments: format!("ls-tree --name-only -z {tree}"),
            code: listed.code,
            stderr: String::from_utf8_lossy(&listed.stderr).into_owned(),
        });
    }
    return Ok(diff_paths(&listed.stdout));
}

/// What:
///  This worktree's own Git directory,
/// read from the `.git` entry at its root.
/// Why:
///   The command runs at the worktree's root (the wrapper refuses any other directory),
/// and `.git` there is the directory itself, or a file naming one for a linked worktree:
///  reading it costs one file and no Git start, which is what a record of another branch must cost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entry = join(cwd, '.git'); const meta = statSync(entry); if (meta.isDirectory()) return entry; const named = readFileSync(entry,'utf8').trim().slice('gitdir: '.length); return resolve(cwd, named);
/// ```
pub(crate) fn the_git_directory_of(git: &PushGit) -> Result<Option<std::path::PathBuf>, String> {
    let entry: std::path::PathBuf = git.cwd.join(".git");
    match std::fs::metadata(entry.as_path()) {
        Ok(meta) if meta.is_dir() => return Ok(Some(entry)),
        Ok(_) => {}
        // What: a `.git` that cannot be read is a read that failed, and never "this is no repository".
        // Why: the sweep's answer lets a command go on, so a state that could not be read must not be taken for a
        // state that needs nothing
        //  (a review of commit `7aef6368e7`, its finding 2).
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            return Err(format!("this worktree's .git could not be read: {error}"));
        }
        Err(_) => return Ok(None),
    }
    let text: String = std::fs::read_to_string(entry.as_path()).map_err(|error| format!("this worktree's .git could not be read: {error}"))?;
    let Some(named) = text.trim().strip_prefix("gitdir: ") else {
        return Err(format!("this worktree's .git does not name a Git directory: {}", entry.display()));
    };
    let directory: std::path::PathBuf = std::path::PathBuf::from(named);
    return Ok(Some(if directory.is_absolute() { directory } else { git.cwd.join(directory) }));
}

/// What:
///  The branch this worktree shows,
/// read from the `HEAD` file of its own Git directory.
/// Why:
///   A record of another branch is another worktree's, and a command in a repository with a record must not
/// start a Git just to learn that
///  (a review of commit `4ca7aebcdb`, its finding 4).
///   Whether this checkout is the one a move is for decides whether an absent index entry is the move's step or a
/// person's removal, which is a question on a command's path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const head = readFileSync(join(gitDir, 'HEAD'), 'utf8').trim(); return head.startsWith('ref: ') ? head.slice(5) : undefined;
/// ```
pub(crate) fn the_branch_this_worktree_shows(git_dir: &std::path::Path) -> Result<Option<String>, String> {
    let path: std::path::PathBuf = git_dir.join("HEAD");
    let head: String = std::fs::read_to_string(path.as_path()).map_err(|error| format!("{} could not be read: {error}", path.display()))?;
    let Some(named) = head.trim().strip_prefix("ref: ") else {
        // A detached HEAD names no branch, which is an answer.
        return Ok(None);
    };
    return Ok(if named.starts_with("refs/heads/") { Some(String::from(named)) } else { None });
}

/// What:
///  The paths of `paths` that still hold exactly what the old tip held,
/// which is this record's own leftover and not a change of the person's.
/// Why:
///   The record names the tip the branch moved from, and that is what tells a leftover of the move
/// from an edit: a path whose stored entry is the old tip's is one the move wrote or one it had not
///  written yet, and the branch's entry belongs there;
/// a path that holds neither the old tip's entry nor the merge's is a person's and stays theirs
///  (a review of commit `cae78e1d7e`, its findings 2 and 3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// paths.filter((path) => !movedOn.includes(path)) // movedOn = git diff --name-only <local> -- <paths>
/// ```
pub(crate) async fn the_paths_still_holding_the_old_tip_awaiting(
    git: &PushGit,
    local: &str,
    paths: &[Vec<u8>],
) -> Result<Vec<Vec<u8>>, SingleFlightError> {
    let arguments: Vec<OsString> = naming(paths, argv(&["--no-optional-locks", "diff", "--name-only", "-z", "--no-renames", local, "--"]));
    let listed = run_query_awaiting(git, &arguments).await?;
    if listed.code != Some(0) {
        return Err(SingleFlightError::Git {
            arguments: format!("diff --name-only -z --no-renames {local}"),
            code: listed.code,
            stderr: String::from_utf8_lossy(&listed.stderr).into_owned(),
        });
    }
    let moved_on: Vec<Vec<u8>> = diff_paths(&listed.stdout);
    let mut still: Vec<Vec<u8>> = Vec::new();
    for path in paths {
        if !moved_on.contains(path) {
            still.push(path.clone());
        }
    }
    return Ok(still);
}

/// What:
///  The paths of `paths` whose index entry is the entry `tree` holds.
/// Why:
///   A path the branch moved from may be stale in the index or in the file, and the write that brings it
/// forward is one or the other:
///  an index that still holds the moved-from entry is written together with the file, and an index already
/// at the branch's entry is written in the file alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// paths.filter((path) => !diffPaths((await git(['--no-optional-locks','diff-index','--cached','--name-only','-z','--no-renames',tree,'--',...paths])).stdout).includes(path))
/// ```
pub(crate) async fn the_paths_whose_index_holds_awaiting(
    git: &PushGit,
    tree: &str,
    paths: &[Vec<u8>],
) -> Result<Vec<Vec<u8>>, SingleFlightError> {
    let arguments: Vec<OsString> = naming(
        paths,
        argv(&["--no-optional-locks", "diff-index", "--cached", "--name-only", "-z", "--no-renames", tree, "--"]),
    );
    let listed = run_query_awaiting(git, &arguments).await?;
    if listed.code != Some(0) {
        return Err(SingleFlightError::Git {
            arguments: format!("diff-index --cached --name-only -z --no-renames {tree}"),
            code: listed.code,
            stderr: String::from_utf8_lossy(&listed.stderr).into_owned(),
        });
    }
    let differing: Vec<Vec<u8>> = diff_paths(&listed.stdout);
    let mut holding: Vec<Vec<u8>> = Vec::new();
    for path in paths {
        if !differing.contains(path) {
            holding.push(path.clone());
        }
    }
    return Ok(holding);
}

/// What:
///  Write the branch's own entry over the paths whose content is the old tip's,
/// path by path, by applying the patch between the two trees to the file alone.
/// Why:
///   Those bytes are the move's own, by the record's old tip, so writing the branch's entry there
/// overwrites nothing a person made;
///  and this write is an act that refuses: `git apply` compares each path's own content with the patch
/// it carries before it writes that path,
/// so a person's edit or a staged change made after the paths were looked at is refused rather than
///  replaced, which `git checkout <branch> -- <paths>` did
///  (a review of commit `4ca7aebcdb`, its finding 1).
///  It writes no index entry, because the index already holds the branch's own entries on these paths:
/// that is what made this the case of a checkout whose index agrees and whose files do not.
///  A refusal is Git's own words, and it names the path it refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const path of paths) await git(['apply','--whitespace=nowarn'], { input: await git(['diff','--binary',old,merge,'--',path]) });
/// ```
pub(crate) async fn the_branchs_own_entries_written_awaiting(
    git: &PushGit,
    old_oid: &str,
    merge_oid: &str,
    in_the_index_too: &[Vec<u8>],
    in_the_file_only: &[Vec<u8>],
) -> Result<Vec<String>, SingleFlightError> {
    let mut refusals: Vec<String> = Vec::new();
    let mut asked: Vec<(&[Vec<u8>], bool)> = vec![(in_the_index_too, true), (in_the_file_only, false)];
    for (paths, also_the_index) in asked.drain(..) {
        for path in paths {
            let arguments: Vec<OsString> = naming(std::slice::from_ref(path), argv(&["--no-optional-locks", "diff", "--binary", old_oid, merge_oid, "--"]));
            let patch = run_query_awaiting(git, &arguments).await?;
            if patch.code != Some(0) {
                refusals.push(format!("the patch between the two tips could not be read: {}", crate::auto_push_reconcile_branch_advance::refusal_of(&patch.stderr)));
                continue;
            }
            if patch.stdout.is_empty() {
                // The path holds the same content in both trees, so there is nothing this move wrote there.
                continue;
            }
            // What: the patch applied to the file, and to the index as well where the index still holds the
            // moved-from entry.
            // Why: `git apply --index` refuses unless the index holds the patch's own pre-image, which is the
            // second proof this write needs, and it takes Git's index lock for the write.
            let apply_arguments: Vec<OsString> = if also_the_index {
                argv(&["apply", "--index", "--whitespace=nowarn"])
            } else {
                argv(&["apply", "--whitespace=nowarn"])
            };
            let applied = crate::auto_push_single_flight::run_query_with_input_awaiting(git, &apply_arguments, patch.stdout.clone()).await?;
            if applied.code != Some(0) {
                refusals.push(crate::auto_push_reconcile_branch_advance::refusal_of(&applied.stderr));
                continue;
            }
            crate::inner_git_words::record_streams(&["apply"], &applied.stdout, &applied.stderr);
        }
    }
    return Ok(refusals);
}
