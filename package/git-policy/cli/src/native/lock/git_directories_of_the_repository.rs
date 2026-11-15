//! What:
//!  The Git directories of one repository,
//!  as the wrapper was told them,
//!  and the answer to "is this folder a Git directory?" that a walk and a watch ask.
//! Why:
//!  The walk that finds a lock and the watch that records one both have to know which folders are
//!  Git directories,
//!  because the object store and the reflog folders directly below one hold no lock of the
//!  worktree's state and are skipped.
//!  A folder cannot say what it is by what it holds or what it is called:
//!  under `refs/` a person chooses the names,
//!  and Git accepts a reference called `refs/heads/marker/HEAD`,
//!  so a `HEAD` file in a reference folder is a reference and not a Git directory
//!  (a review of `ca936e0b93`,
//! finding 2).
//!  What the wrapper knows instead is what Git told it,
//!  through the invocation's identity and through Git's own layout in a place the wrapper was told:
//!
//!  - the Git directory of the worktree the command runs in, and the common Git directory
//!    (`roots_of`,
//! `common_dir_for`);
//!  - every linked worktree's administrative directory,
//!    which Git registers as one entry of the common directory's own `worktrees` folder;
//!  - every submodule's Git directory,
//!    which Git keeps as one entry of a Git directory's own `modules` folder,
//!    a submodule of a submodule one level further down.
//!
//!  The two folders of the second and third kind are Git's own layout at a place that is already a
//!  Git directory:
//!  nothing under `refs/` is ever read this way,
//!  because `refs` is not a Git directory and its folders are not asked.
//!  Every other folder is a Git directory only when this list names it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const gitDirectories = gitDirectoriesOf([gitDir, commonDir], commonDir); // [gitDir, commonDir, ...worktrees/*, ...modules/*]
//! ```

/// A `PathBuf` is an owned filesystem path,
/// and a `Path` a borrowed one.
use std::path::{Path, PathBuf};

/// The folder of a common Git directory that holds each linked worktree's registration.
/// Why the name is read here and never under `refs/`:
///  Git itself defines this folder below a common Git directory
///  (`git worktree add` writes one entry in it, and `git worktree list` reads them),
///  and a person cannot choose it there.
///  A reference namespace of the same name is below `refs/`,
///  where no name is read as anything but a person's own.
const LINKED_WORKTREES_FOLDER: &str = "worktrees";

/// The folder of a Git directory that holds each submodule's Git directory.
/// Why:
///  Git keeps a submodule's Git directory below its superproject's,
///  one entry per submodule,
///  and a person cannot choose that name there.
const SUBMODULES_FOLDER: &str = "modules";

/// How many levels of nested Git directories are read,
/// so a repository shaped like a tree of submodules cannot grow a command.
/// Why:
///  A submodule of a submodule keeps its Git directory one level further down
///  (`<git-dir>/modules/<name>/modules/<name>`),
///  and three levels is far more than any repository of this machine holds.
///  Git directories the bound leaves out are walked as ordinary folders:
///  what that costs is watch slots and candidate locks the proof refuses,
///  never a wrong decision.
const MOST_LEVELS: usize = 3;

/// What:
///  The Git directories of the repository whose roots are `roots`,
///  each of which the caller already knows is a Git directory,
///  with the common directory `common_dir`.
/// Why:
///  This is the one list the walk and the watch decide by,
///  so a name a person can choose is never read as a folder's kind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gitDirectoriesOf(roots, commonDir) // roots, then worktrees/* and modules/* to three levels
/// ```
pub fn git_directories_of(roots: &[PathBuf], common_dir: &Path) -> Vec<PathBuf> {
    let mut directories: Vec<PathBuf> = roots.to_vec();
    // The entries of Git's two layout folders are Git directories themselves, and each of them has the two layout
    //  folders of its own; the loop reads them one level at a time, bounded.
    for _level in 0..MOST_LEVELS {
        let mut found: Vec<PathBuf> = Vec::new();
        for directory in directories.iter() {
            for child in entries_of(directory.as_path(), SUBMODULES_FOLDER) {
                found.push(child);
            }
        }
        for child in entries_of(common_dir, LINKED_WORKTREES_FOLDER) {
            found.push(child);
        }
        found.sort();
        found.dedup();
        let before: usize = directories.len();
        for child in found {
            if !directories.contains(&child) {
                directories.push(child);
            }
        }
        if directories.len() == before {
            break;
        }
    }
    directories.sort();
    directories.dedup();
    return directories;
}

/// What:
///  The common Git directory of the Git directory `git_dir`,
///  or `git_dir` itself when it is its own common directory.
/// Why:
///  A linked worktree's administrative directory says where the common one is in Git's own `commondir` file
///  (`repo_read/git_directory_roots.rs`,
/// `common_dir_for`),
///  and a walk or a watch that has only a Git directory still has to read Git's layout below the common
///  one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// commonDirOf(gitDir) // readFileSync(gitDir/commondir) ? resolve(...) : gitDir
/// ```
pub fn common_dir_of(git_dir: &Path) -> PathBuf {
    return crate::git_directory_roots::common_dir_for(git_dir);
}

/// What:
///  Whether `git_directories` names this folder,
///  which is the whole of what the walk and the watch ask about a folder.
/// Why:
///  One answer, so the two cannot disagree about which folders are Git directories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isAGitDirectory(folder) // gitDirectories.includes(folder)
/// ```
pub fn is_a_git_directory(git_directories: &[PathBuf], folder: &Path) -> bool {
    return git_directories.iter().any(|named| { return named == folder; });
}

/// What:
///  The directories inside `folder/layout`, in order,
///  or nothing when there is no such folder.
/// Why:
///  Git's layout folders hold one directory per registration,
///  and a repository with none has no such folder at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// entriesOf(gitDir, 'modules') // [gitDir/modules/a, gitDir/modules/b]
/// ```
fn entries_of(folder: &Path, layout: &str) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let Ok(entries) = std::fs::read_dir(folder.join(layout)) else {
        return found;
    };
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_dir() {
            found.push(entry.path());
        }
    }
    found.sort();
    return found;
}

/// Controls of the list itself stay out of the release executable.
#[cfg(test)]
#[path = "git_directories_of_the_repository_tests.rs"]
mod tests;
