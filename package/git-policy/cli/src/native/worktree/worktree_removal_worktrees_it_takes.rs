//! What:
//!  The worktrees one removal takes:
//!  the one it names,
//!  and every worktree this repository registers inside it,
//!  each with its own Git directory.
//! Why:
//!  A removal deletes the files below the directory it names,
//!  and that directory holds the checkout and the own Git directory of every worktree registered inside it,
//!  so the removal takes those too.
//!  Every question a guard asks before a removal is a question about what that removal takes,
//!  and a question asked of the outer directory alone misses the Git work of every worktree inside it:
//!  a hidden edit in such a worktree went with the outer one while the same edit in the outer directory was refused
//!  (a review of `8858f843ee`, finding 2).
//!  So the list is made once, here, and every question of every route goes over it
//!  (`git_work_a_removal_would_strand`, `hidden_edits_beneath`).
//!  The registrations are read strictly: one `gitdir` that cannot be read may name a worktree inside the one being
//!  removed, so it is the reason nothing is removed rather than an entry quietly passed over.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const taken = theWorktreesItTakes({ commonDir, target, admin }); // [{ root, admin }], or throws the reason
//! ```

use crate::worktree_landed_registrations::registrations_a_removal_takes;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};

/// What:
///  One thing a removal takes:
///  a worktree of this repository, with its own Git directory,
///  or a Git repository inside the worktree that this repository does not register.
/// Why:
///   Both hold Git work that goes with the removal,
///  and the questions a guard asks are asked of every one of them.
///  The second kind is the repository a person cloned inside an ignored folder:
///  its commits and branches are its own, and its Git directory goes with the directory the removal deletes
///  (a review of `746040080c`, finding 3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Taken = { kind: 'worktree'; root: string; admin: string } | { kind: 'repository'; root: string; gitDir: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Taken {
    /// A worktree this repository registers,
    /// with the Git directory that holds its own state.
    Worktree {
        /// The worktree's own directory.
        root: PathBuf,
        /// Its own Git directory,
        /// below the common directory.
        admin: PathBuf,
    },
    /// A Git repository inside the worktree that this repository does not register,
    /// with the directory its own records are in.
    Repository {
        /// The directory holding the repository's `.git`.
        root: PathBuf,
        /// Its Git directory,
        /// which the removal deletes with it.
        git_dir: PathBuf,
    },
}

/// What:
///  The things a removal of `target` takes:
///  the target,
///  every worktree this repository registers inside it,
///  and every repository inside the directories it ignores,
///  or the reason one of them cannot be read.
/// Why:
///   One list,
///  made once,
///  is what keeps every question of a removal about the same set:
///  a question that walked its own list would keep or miss a worktree the others do not know about.
///  The worktrees come first, so a report about the target reads first.
///  The look for repositories is the ignored tree the removal is about to delete,
///  and it answers with a reason when it cannot finish, because nothing then proves there is none
///  (`repositories_inside_ignored_directories`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theThingsARemovalTakes({ context, commonDir, target, admin }): Taken[]; // throws the reason
/// ```
///
/// # Errors
///
/// The words for a registration that cannot be listed or read,
/// and for a look at the ignored tree that could not finish.
pub(crate) fn the_things_a_removal_takes(
    context: &crate::transaction_git::GitContext,
    common_dir: &Path,
    target: &Path,
    admin: &Path,
) -> Result<Vec<Taken>, String> {
    let mut taken: Vec<Taken> = vec![Taken::Worktree { root: target.to_path_buf(), admin: admin.to_path_buf() }];
    for one in registrations_a_removal_takes(common_dir, target)? {
        taken.push(Taken::Worktree { root: one.worktree, admin: one.admin });
    }
    // The look starts a Git that reads the worktree's index, and a worktree whose index is no plain file is one this
    // program must not hand to a Git at all: the question about hidden edits refuses it with its own words, and the look
    // would only wait for a writer that may never come (`SPEC.md`, "Every wait has a bound";
    // a review of `528279d099`, finding 2). So no look is made where the index is there and is no plain file.
    let index: PathBuf = admin.join("index");
    let index_is_readable: bool = match std::fs::symlink_metadata(index.as_path()) {
        Err(_absent) => true,
        Ok(metadata) => metadata.is_file(),
    };
    if !index_is_readable {
        return Ok(taken);
    }
    for relative in crate::worktree_removal_takes::repositories_inside_ignored_directories(context, target, admin)? {
        let root: PathBuf = target.join(std::ffi::OsString::from_vec(relative.clone()));
        if root == target {
            continue;
        }
        let git_dir: PathBuf = repository_git_directory(root.as_path())?;
        taken.push(Taken::Repository { root, git_dir });
    }
    return Ok(taken);
}

/// What:
///  The Git directory of the repository whose working tree is `root`,
///  or the reason it cannot be named.
/// Why:
///   A `.git` is a directory of the repository's own,
///  a file that names one elsewhere (a worktree of that other repository),
///  or a link;
///  and a name this program cannot read is the reason the removal stops, because it may hold a repository's Git work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function repositoryGitDirectory(root): string; // throws the reason
/// ```
fn repository_git_directory(root: &Path) -> Result<PathBuf, String> {
    let pointer: PathBuf = root.join(".git");
    let metadata: std::fs::Metadata = std::fs::metadata(pointer.as_path())
        .map_err(|error| return format!("the repository record {} cannot be read ({error})", pointer.display()))?;
    if metadata.is_dir() {
        return std::fs::canonicalize(pointer.as_path()).map_err(|error| return format!("the Git directory of {} cannot be resolved ({error})", root.display()));
    }
    let text: Vec<u8> = std::fs::read(pointer.as_path()).map_err(|error| return format!("the repository record {} cannot be read ({error})", pointer.display()))?;
    let line: &[u8] = text.trim_ascii();
    let Some(named) = line.strip_prefix(b"gitdir: ") else {
        return Err(format!("the repository record {} names no Git directory", pointer.display()));
    };
    let spelled: PathBuf = PathBuf::from(std::ffi::OsString::from_vec(named.to_vec()));
    let resolved: PathBuf = if spelled.is_absolute() { spelled } else { root.join(spelled) };
    return std::fs::canonicalize(resolved.as_path())
        .map_err(|error| return format!("the Git directory {} names cannot be resolved ({error})", pointer.display()));
}

/// The list of what a removal takes stays out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_worktrees_it_takes_tests.rs"]
mod tests;
