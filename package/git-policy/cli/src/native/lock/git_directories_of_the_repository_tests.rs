//! What:
//!  Controls of the list of Git directories a walk and a watch decide by:
//!  what the wrapper was told,
//!  and never what a folder holds or is called under `refs/`.
//! Why:
//!  A folder classified by its contents made a reference named `HEAD` hide the `objects` namespace
//!  beside it (a review of `ca936e0b93`,
//! finding 2).
//!  The decision is a lookup now,
//!  so the controls state three things:
//!  Git's own layout below a Git directory is read,
//!  a name a person can choose under `refs/` is not,
//!  and nested submodules are read to the bound and no further.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(gitDirectoriesOf([root], root)).toEqual([root, root/worktrees/held, root/modules/sub]);
//! expect(isAGitDirectory(list, root/refs/heads/marker)).toBe(false);
//! ```

use super::{MOST_LEVELS, git_directories_of, is_a_git_directory};
use std::path::{Path, PathBuf};

/// What:
///  A folder made as `path`,
///  with its parents,
///  and nothing in it.
/// Why:
///  Every control below states a layout of folders only, which is what the list is read from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkdirSync(path, { recursive: true })
/// ```
fn a_folder(path: &Path) {
    std::fs::create_dir_all(path).expect("a folder of the fixture layout");
}

/// What:
///  A file made at `path`,
///  with its parents.
/// Why:
///  A reference is a file, and a control states that a file named `HEAD` in a reference folder is a
///  reference and not a Git directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkdirSync(dirname(path), { recursive: true }); writeFileSync(path, '')
/// ```
fn a_file(path: &Path) {
    a_folder(path.parent().expect("a file in a folder"));
    std::fs::write(path, b"").expect("a file of the fixture layout");
}

/// What:
///  The Git directories of a repository are the two the invocation resolved and the entries of Git's
///  own layout folders below them,
///  and a reference folder of any name is not one of them.
/// Why:
///  The object store and the reflog folders are skipped only below the Git directories of this list,
///  and a person chooses every name under `refs/`:
///  a `.git/refs/heads/marker/HEAD` written here is a reference Git accepts,
///  and `marker` is a reference folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stateTheLayout(); const list = gitDirectoriesOf([root, root], root);
/// expect(list).toEqual([root, root/modules/sub, root/worktrees/held]); expect(isAGitDirectory(list, root/refs/heads/marker)).toBe(false)
/// ```
#[test]
fn git_directories_are_the_ones_the_wrapper_was_told_and_gits_own_layout_below_them() {
    let root: PathBuf = crate::test_support::fixture("git-directories-layout");
    a_folder(&root.join("refs/heads/marker/objects"));
    a_file(&root.join("refs/heads/marker/HEAD"));
    a_folder(&root.join("refs/heads/marker/modules"));
    a_folder(&root.join("refs/heads/marker/worktrees/held"));
    a_folder(&root.join("worktrees/held"));
    a_folder(&root.join("modules/sub"));
    let directories: Vec<PathBuf> = git_directories_of(&[root.clone(), root.clone()], root.as_path());
    let marker: PathBuf = root.join("refs/heads/marker");
    let found: bool = is_a_git_directory(directories.as_slice(), marker.as_path());
    let held: bool = is_a_git_directory(directories.as_slice(), root.join("worktrees/held").as_path());
    let submodule: bool = is_a_git_directory(directories.as_slice(), root.join("modules/sub").as_path());
    let listed: Vec<PathBuf> = directories.clone();
    crate::test_support::remove(root.as_path());
    assert!(held, "a linked worktree's registration is a Git directory: {listed:?}");
    assert!(submodule, "a submodule's Git directory is a Git directory: {listed:?}");
    assert!(!found, "a reference folder holding a file named HEAD is not a Git directory: {listed:?}");
    assert!(!listed.contains(&marker.join("objects")), "a reference namespace named objects is not a Git directory: {listed:?}");
    assert!(!listed.contains(&marker.join("modules")), "a reference namespace named modules is not a Git directory: {listed:?}");
}

/// What:
///  A submodule of a submodule is followed to `MOST_LEVELS` and no further,
///  and a folder deeper than that is walked as an ordinary folder.
/// Why:
///  The bound is what keeps a repository shaped like a tree of submodules from growing a command;
///  a Git directory the bound leaves out costs watch slots and refuses no lock it should clear.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// nestModules(MOST_LEVELS + 1); expect(list).toContain(deepestWithinBound); expect(list).not.toContain(deeper)
/// ```
#[test]
fn the_submodule_levels_stop_at_the_bound() {
    let root: PathBuf = crate::test_support::fixture("git-directories-levels");
    let mut deepest_within: PathBuf = root.clone();
    let mut beyond: PathBuf = root.clone();
    for _level in 0..MOST_LEVELS {
        deepest_within = deepest_within.join("modules/sub");
        beyond = deepest_within.clone();
    }
    beyond = beyond.join("modules/sub");
    a_folder(&beyond);
    let directories: Vec<PathBuf> = git_directories_of(&[root.clone()], root.as_path());
    let within: bool = is_a_git_directory(directories.as_slice(), deepest_within.as_path());
    let beyond_named: bool = is_a_git_directory(directories.as_slice(), beyond.as_path());
    crate::test_support::remove(root.as_path());
    assert!(within, "the last submodule inside the bound is a Git directory");
    assert!(!beyond_named, "a submodule deeper than the bound is walked as an ordinary folder");
}
