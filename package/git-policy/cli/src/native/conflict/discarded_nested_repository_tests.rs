//! What:
//!  The move of the nested repositories a clean would delete:
//!  every byte and the repository's own `.git`,
//!  the place below the saved copies,
//!  and every refusal that keeps the worktree as it was.
//! Why:
//!  The folder of saved copies is the only place a nested repository can be found after the clean,
//!  so the path below it must hold the whole directory and the audit line must name that exact path.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const [copy] = setNestedRepositoriesAside(common, worktree, [Buffer.from('nested')], 1_000);
//! // expect(existsSync(join(copy.savedTo, '.git', 'HEAD'))).toBe(true);
//! ```

use super::{describe_rename_failure, set_nested_repositories_aside};
use crate::command_test_support::{fixture, remove};
use crate::conflict_sides::{SAVED_DIRECTORY, SavedCopy, SaveFailure};
use std::path::{Path, PathBuf};

/// What:
///  A worktree holding one nested repository and the common Git directory beside it.
/// Why:
///   The save moves a directory from the first into the second,
///  so a control needs both and nothing else.
struct Places {
    /// The fixture root the whole control is removed with.
    root: PathBuf,
    /// The worktree top level Git printed the paths from.
    worktree: PathBuf,
    /// The common Git directory the saved copies live under.
    common: PathBuf,
}

fn places(name: &str) -> Places {
    let root: PathBuf = fixture(name);
    let worktree: PathBuf = root.join("linked");
    let common: PathBuf = root.join("main/.git");
    std::fs::create_dir_all(worktree.join("nested/.git")).expect("a nested repository");
    std::fs::write(worktree.join("nested/tracked.txt"), b"modified user data\n").expect("a modified tracked file");
    std::fs::write(worktree.join("nested/untracked.txt"), b"untracked user data\n").expect("an untracked file");
    std::fs::write(worktree.join("nested/.git/HEAD"), b"ref: refs/heads/inner\n").expect("the repository's own history");
    std::fs::create_dir_all(common.as_path()).expect("the common Git directory");
    return Places { root, worktree, common };
}

/// The repository moves whole below the saved copies,
/// and the answer names its path below the worktree and the place it now holds.
#[test]
fn a_repository_moves_whole_below_the_saved_copies() {
    let found: Places = places("discarded-nested-repository");
    let copies: Vec<SavedCopy> = set_nested_repositories_aside(
        found.common.as_path(),
        found.worktree.as_path(),
        &[b"nested".to_vec()],
        1_000,
    )
    .expect("the move");
    assert!(!found.worktree.join("nested").exists(), "the repository is still in the worktree");
    let copy: &SavedCopy = copies.first().expect("one copy");
    assert_eq!(copy.path, "nested");
    let place: &Path = copy.saved_to.as_deref().expect("a place");
    let saved_root: PathBuf = found.common.join(SAVED_DIRECTORY);
    assert!(place.starts_with(saved_root.as_path()), "{}", place.display());
    assert_eq!(std::fs::read_to_string(place.join("tracked.txt")).expect("the tracked file"), "modified user data\n");
    assert_eq!(
        std::fs::read_to_string(place.join("untracked.txt")).expect("the untracked file"),
        "untracked user data\n"
    );
    assert_eq!(
        std::fs::read_to_string(place.join(".git/HEAD")).expect("the repository's own history"),
        "ref: refs/heads/inner\n"
    );
    remove(found.root.as_path());
}

/// Two repositories of one clean share the one directory named for that save,
///  so a person finds them together and the limits count them once.
#[test]
fn two_repositories_share_one_saved_directory() {
    let found: Places = places("discarded-nested-repository-two");
    std::fs::create_dir_all(found.worktree.join("vendor/lib/.git")).expect("a second repository");
    let copies: Vec<SavedCopy> = set_nested_repositories_aside(
        found.common.as_path(),
        found.worktree.as_path(),
        &[b"nested".to_vec(), b"vendor/lib".to_vec()],
        2_000,
    )
    .expect("the move");
    assert_eq!(copies.len(), 2, "{copies:?}");
    let first: &Path = copies.first().expect("the first copy").saved_to.as_deref().expect("a place");
    let second: &Path = copies.get(1).expect("the second copy").saved_to.as_deref().expect("a place");
    let saved_root: PathBuf = found.common.join(SAVED_DIRECTORY);
    let stamp_of = |place: &Path| -> Option<std::ffi::OsString> {
        return Some(place.strip_prefix(saved_root.as_path()).ok()?.components().next()?.as_os_str().to_os_string());
    };
    assert_eq!(stamp_of(first), stamp_of(second), "one directory for one save");
    assert!(stamp_of(first).is_some(), "the place is below the saved copies: {}", first.display());
    assert!(!found.worktree.join("nested").exists(), "the first repository is still in the worktree");
    assert!(second.ends_with("vendor/lib"), "{}", second.display());
    remove(found.root.as_path());
}

/// What:
///  A path that leaves the worktree is refused and nothing moves,
///  and a repository that is no longer there is refused too.
/// Why:
///   The paths come from Git,
///  and a path this reader misread must cost a refusal,
///  never a directory outside the worktree.
#[test]
fn a_refused_move_changes_nothing() {
    let found: Places = places("discarded-nested-repository-refused");
    for (repositories, expected) in [
        (vec![b"../outside".to_vec()], "../outside"),
        (vec![b"missing".to_vec()], "missing"),
    ] {
        let failure: SaveFailure = set_nested_repositories_aside(
            found.common.as_path(),
            found.worktree.as_path(),
            repositories.as_slice(),
            3_000,
        )
        .expect_err("a refused move");
        assert_eq!(failure.subject, expected);
        assert!(found.worktree.join("nested").exists(), "a refused move lost the repository it did not move");
    }
    remove(found.root.as_path());
}

/// A first move that succeeds stays saved when a later one fails,
///  and the place it went is the one directory that save made.
#[test]
fn a_later_move_that_fails_keeps_the_earlier_one_saved() {
    let found: Places = places("discarded-nested-repository-later");
    let failure: SaveFailure = set_nested_repositories_aside(
        found.common.as_path(),
        found.worktree.as_path(),
        &[b"nested".to_vec(), b"missing".to_vec()],
        4_000,
    )
    .expect_err("the second move fails");
    assert_eq!(failure.subject, "missing");
    assert!(!found.worktree.join("nested").exists(), "the first move is not undone");
    let saved_root: PathBuf = found.common.join(SAVED_DIRECTORY);
    let names: Vec<String> = std::fs::read_dir(saved_root.as_path())
        .expect("the saved copies")
        .map(|entry| { return entry.expect("an entry").file_name().to_string_lossy().into_owned(); })
        .collect();
    assert_eq!(names.len(), 1, "{names:?}");
    let stamp: PathBuf = saved_root.join(names.first().expect("one directory"));
    assert!(stamp.join("nested/.git/HEAD").exists(), "{}", stamp.display());
    remove(found.root.as_path());
}

/// A repository on another file system is refused in words that say why,
///  because the wrapper does not copy a whole repository.
#[test]
fn another_file_system_is_refused_in_words_a_person_can_act_on() {
    let failure: String = describe_rename_failure(std::io::Error::from_raw_os_error(libc::EXDEV));
    assert!(failure.contains("another file system"), "{failure}");
    assert!(failure.contains("does not copy a whole repository"), "{failure}");
}
