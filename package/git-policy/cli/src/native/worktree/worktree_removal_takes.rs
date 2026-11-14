//! What:
//!  What a removal deletes besides the files Git tracks,
//!  named for the person who reads its line:
//!  the directories Git ignores and the Git repositories inside them.
//! Why:
//!   A removal deletes what Git ignores without a word,
//!  and the owner's decision lets that happen as real Git does,
//!  with each thing that goes logged
//!  (`doc/handover/cli-git-rust-implementation.md`,
//!  "How far the protection of a worktree being removed reaches").
//!  A nested repository is the case that costs the most:
//!  its commits can exist nowhere else,
//!  and nobody learns their names once the directory is gone,
//!  so the line names the repository's path and the tip of every branch it holds.
//!  The reason the sweep prints a directory unread is that reading every file below one would cost the whole tree on every
//!  merge;
//!  listing them is one Git start and no file read,
//!  and it is made only for a worktree that is about to go.
//!  Nothing here fails a removal:
//!  what cannot be read is what the line leaves unsaid,
//!  and the deletion itself is the removal's own business.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const went = whatTheRemovalTakes(context, worktree, admin); // string[] lines
//! ```

use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use crate::worktree_ladder_target::checkout_environment;
use std::collections::BTreeSet;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// The name every Git repository keeps its own records in,
/// as a directory or as a file that points at them.
const GIT_POINTER: &str = ".git";

/// The separator Git puts between the components of a path.
const SEPARATOR: u8 = b'/';

/// How many directories one line names before it counts the rest.
const SHOWN_DIRECTORIES: usize = 3;

/// How many repositories one line names before it counts the rest.
const SHOWN_REPOSITORIES: usize = 4;

/// How many branches of one repository a line names before it counts the rest.
const SHOWN_BRANCHES: usize = 3;

/// How many digits of a commit a line shows,
/// which is the prefix Git prints beside a commit with room to be unambiguous.
const SHOWN_DIGITS: usize = 12;

/// How many entries of an ignored directory this reads while it looks for repositories.
/// Why:
///   A removal of a worktree with a large ignored tree already walks it for its own proofs,
///  and a bound keeps a directory holding more than this from costing the merge a longer walk than the removal itself.
const WALKED_ENTRIES: usize = 50_000;

/// What:
///  What a removal of `worktree` deletes besides its tracked files,
///  in the words of the line that reports the removal.
/// Why:
///   The line has to name what went,
///  and the two things a person looks for afterwards are the build output they kept below an ignored name and a
///  repository they cloned inside one.
///  The listing is one Git start over the paths Git already knows are ignored,
///  so nothing is read that the removal would not delete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const went = whatTheRemovalTakes(context, worktree, admin);
/// ```
pub(crate) fn what_the_removal_takes(context: &GitContext, worktree: &Path, admin: &Path) -> Vec<String> {
    let ignored: Vec<Vec<u8>> = match ignored_paths(context, worktree, admin) {
        Ok(found) => found,
        // Nothing is proven about what goes, and the removal is the caller's own business: the line says nothing rather
        // than something untrue.
        Err(_reason) => return Vec::new(),
    };
    let mut lines: Vec<String> = Vec::new();
    let repositories: BTreeSet<Vec<u8>> = repositories_beneath(worktree, ignored.as_slice());
    let mut directories: BTreeSet<Vec<u8>> = directories_of(ignored.as_slice());
    directories.retain(|name| return !repositories.contains(name));
    if !directories.is_empty() {
        lines.push(directories_line(directories));
    }
    lines.extend(repository_lines(context, worktree, repositories));
    return lines;
}

/// What:
///  Every path Git ignores in the worktree,
///  each as Git names it.
/// Why:
///   Git's own ignore rules are what the removal deletes by,
///  so they are what names the directories and the repositories that go.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ignoredPaths(context, worktree, admin) // [Uint8Array]
/// ```
fn ignored_paths(context: &GitContext, worktree: &Path, admin: &Path) -> Result<Vec<Vec<u8>>, String> {
    let arguments: Vec<OsString> = ["ls-files", "--others", "--ignored", "--exclude-standard", "-z"].map(OsString::from).to_vec();
    let request: GitRequest = GitRequest {
        cwd: worktree.to_path_buf(),
        arguments,
        without_prefix: true,
        environment: checkout_environment(admin, worktree),
        ..GitRequest::default()
    };
    let output: GitOutput = run_git(context, &request).map_err(|error| return format!("the ignored paths of {} cannot be asked of Git ({error})", worktree.display()))?;
    if output.code != Some(0) {
        return Err(format!(
            "Git cannot list the ignored paths of {} ({})",
            worktree.display(),
            String::from_utf8_lossy(output.stderr.as_slice()).trim()
        ));
    }
    return Ok(output.stdout.split(|byte: &u8| return *byte == 0).filter(|path: &&[u8]| return !path.is_empty()).map(<[u8]>::to_vec).collect());
}

/// What:
///  The directories every ignored path lies in,
///  each named once.
/// Why:
///   Git lists a whole ignored directory as its files when the caller asks without `--directory`,
///  and a person counts directories rather than the files inside them.
///  A path inside a repository is left to the repository's own line,
///  which names the repository once instead of a directory per file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// directoriesOf(paths) // ['ignored', 'build/.cache']
/// ```
fn directories_of(paths: &[Vec<u8>]) -> BTreeSet<Vec<u8>> {
    let mut names: BTreeSet<Vec<u8>> = BTreeSet::new();
    for path in paths {
        if path.split(|byte| return *byte == SEPARATOR).any(|name| return name == GIT_POINTER.as_bytes()) {
            continue;
        }
        let Some(at) = path.iter().rposition(|byte| return *byte == SEPARATOR) else { continue };
        names.insert(path[..at].to_vec());
    }
    return names;
}

/// What:
///  The line that names the ignored directories which went.
/// Why:
///   A person who kept something below an ignored name reads which name went,
///  and a list longer than a few is counted rather than printed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// directoriesLine(names) // 'the ignored directories `ignored`, `build` and 2 more'
/// ```
fn directories_line(names: BTreeSet<Vec<u8>>) -> String {
    let mut shown: Vec<String> = Vec::new();
    let total: usize = names.len();
    for name in names.into_iter().take(SHOWN_DIRECTORIES) {
        shown.push(format!("`{}`", String::from_utf8_lossy(name.as_slice())));
    }
    let rest: usize = total.saturating_sub(shown.len());
    return match rest {
        0 => format!("the ignored directories {}", shown.join(", ")),
        1 => format!("the ignored directories {} and 1 more", shown.join(", ")),
        _ => format!("the ignored directories {} and {rest} more", shown.join(", ")),
    };
}

/// What:
///  The roots of every Git repository inside the ignored directories a removal deletes.
/// Why:
///   A `.git` entry is the only sign of a repository inside a worktree,
///  and Git never lists one:
///  `ls-files` walks around a nested repository the way it walks around the worktree's own,
///  so the directories Git ignores are read instead.
///  Only those are walked,
///  and only for a worktree that is about to go,
///  so the cost is the ignored tree the removal already deletes and nothing of the tracked one.
///  A repository nested inside another is not looked for below a root already found:
///  the outer repository's line names the directory that takes both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// repositoriesBeneath(worktree, ignored) // ['ignored/clone']
/// ```
fn repositories_beneath(worktree: &Path, paths: &[Vec<u8>]) -> BTreeSet<Vec<u8>> {
    let mut roots: BTreeSet<Vec<u8>> = BTreeSet::new();
    let mut stack: Vec<(Vec<u8>, PathBuf)> = Vec::new();
    for name in top_level_directories(paths) {
        let shown: String = String::from_utf8_lossy(name.as_slice()).into_owned();
        stack.push((name, worktree.join(shown)));
    }
    let mut read: usize = 0;
    while let Some((relative, directory)) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(directory.as_path()) else { continue };
        for entry in entries.flatten() {
            read += 1;
            if read > WALKED_ENTRIES {
                return roots;
            }
            let name: std::ffi::OsString = entry.file_name();
            let mut child: Vec<u8> = relative.clone();
            child.push(SEPARATOR);
            child.extend_from_slice(name.as_encoded_bytes());
            if name == OsStr::new(GIT_POINTER) {
                roots.insert(relative.clone());
                continue;
            }
            if entry.file_type().map(|kind| return kind.is_dir()).unwrap_or(false) {
                stack.push((child, entry.path()));
            }
        }
    }
    return roots;
}

/// What:
///  The first components of the ignored paths,
///  which are the directories a repository inside the worktree can lie in.
/// Why:
///   Every ignored path is below one of them,
///  so walking those covers the whole ignored tree once,
///  and a path that lies in the worktree's root itself names no directory of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// topLevelDirectories(paths) // ['ignored', 'build']
/// ```
fn top_level_directories(paths: &[Vec<u8>]) -> BTreeSet<Vec<u8>> {
    let mut names: BTreeSet<Vec<u8>> = BTreeSet::new();
    for path in paths {
        let Some(at) = path.iter().position(|byte| return *byte == SEPARATOR) else { continue };
        names.insert(path[..at].to_vec());
    }
    return names;
}

/// What:
///  One line for each Git repository the removal deletes,
///  naming its path and the tip of every branch it holds.
/// Why:
///   A repository's commits can exist nowhere else,
///  so the names of its branches and the commits they point at are the last thing a person can look for once the
///  directory is gone.
///  A repository Git reports without a branch says so,
///  because a `HEAD` on nothing is a state its owner has to know about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// repositoryLines(context, roots) // ['the Git repository at `ignored/clone`, whose branches are `main` at 1234567']
/// ```
fn repository_lines(context: &GitContext, worktree: &Path, roots: BTreeSet<Vec<u8>>) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let total: usize = roots.len();
    for root in roots.into_iter().take(SHOWN_REPOSITORIES) {
        lines.push(repository_line(context, worktree, root.as_slice()));
    }
    if total > lines.len() {
        lines.push(format!("{} more Git repositories went with it", total - lines.len()));
    }
    return lines;
}

/// What:
///  The line for one Git repository a removal deletes.
/// Why:
///   The path says where it was,
///  and the branches say which commits it held,
///  which is everything a person needs to look for the work elsewhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// repositoryLine(context, worktree, 'ignored/clone') // 'the Git repository at `ignored/clone`, whose branches are `main` at 1234567'
/// ```
fn repository_line(context: &GitContext, worktree: &Path, root: &[u8]) -> String {
    let shown: String = String::from_utf8_lossy(root).into_owned();
    // The root is named as Git named it, below the worktree, because that is where a person looks for it.
    let path: PathBuf = worktree.join(shown.as_str());
    return match branch_tips(context, path.as_path()) {
        Ok(tips) if tips.is_empty() => format!("the Git repository at `{shown}`, which held no branch, so its commits are named by nothing now"),
        Ok(tips) => format!("the Git repository at `{shown}`, whose branches are {}", tips.join(", ")),
        Err(reason) => format!("the Git repository at `{shown}`, whose branches could not be read ({reason})"),
    };
}

/// What:
///  The branches of the repository at `root`,
///  each as its short name and its commit,
///  up to `SHOWN_BRANCHES` of them.
/// Why:
///   One Git start per repository the removal deletes,
///  and a repository is rare enough that this never costs a removal anything measurable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// branchTips(context, 'ignored/clone') // ['`main` at 1234567']
/// ```
fn branch_tips(context: &GitContext, root: &Path) -> Result<Vec<String>, String> {
    let arguments: Vec<OsString> = [OsString::from("for-each-ref"), OsString::from("--format=%(refname:short)%09%(objectname)"), OsString::from("refs/heads")].to_vec();
    let output: GitOutput = run_git(context, &GitRequest::new(root, arguments.as_slice())).map_err(|error| return format!("{error}"))?;
    if output.code != Some(0) {
        return Err(String::from_utf8_lossy(output.stderr.as_slice()).trim().to_owned());
    }
    let mut tips: Vec<String> = Vec::new();
    let mut counted: usize = 0;
    for line in String::from_utf8_lossy(output.stdout.as_slice()).lines() {
        let Some((name, commit)) = line.split_once('\t') else { continue };
        counted += 1;
        if tips.len() < SHOWN_BRANCHES {
            let short: &str = &commit[..SHOWN_DIGITS.min(commit.len())];
            tips.push(format!("`{name}` at {short}"));
        }
    }
    if counted > tips.len() {
        tips.push(format!("and {} more branches", counted - tips.len()));
    }
    return Ok(tips);
}

/// The controls of this question stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_takes_tests.rs"]
mod tests;
