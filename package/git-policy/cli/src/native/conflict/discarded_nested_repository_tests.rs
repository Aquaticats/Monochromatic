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
use crate::conflict_sides::{SAVED_DIRECTORY, SavedCopy};
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
        let stopped = set_nested_repositories_aside(
            found.common.as_path(),
            found.worktree.as_path(),
            repositories.as_slice(),
            3_000,
        )
        .expect_err("a refused move");
        assert_eq!(stopped.failure.subject, expected);
        assert!(stopped.still_moved.is_empty(), "{:?}", stopped.still_moved);
        assert!(found.worktree.join("nested").exists(), "a refused move lost the repository it did not move");
    }
    remove(found.root.as_path());
}

/// What:
///  A first move that succeeds is put back when a later one fails,
///  so the worktree is exactly as the command found it.
/// Why:
///  A rejection that says "nothing was changed" has to be true,
///  and a repository left below the saved copies is one the sentence hides:
///  the person is told the repository is where it was while it is somewhere else.
#[test]
fn a_later_move_that_fails_puts_the_earlier_one_back() {
    let found: Places = places("discarded-nested-repository-later");
    let stopped = set_nested_repositories_aside(
        found.common.as_path(),
        found.worktree.as_path(),
        &[b"nested".to_vec(), b"missing".to_vec()],
        4_000,
    )
    .expect_err("the second move fails");
    assert_eq!(stopped.failure.subject, "missing");
    assert!(stopped.still_moved.is_empty(), "{:?}", stopped.still_moved);
    assert_eq!(
        std::fs::read_to_string(found.worktree.join("nested/tracked.txt")).expect("the repository is where it was"),
        "modified user data\n"
    );
    assert_eq!(
        std::fs::read_to_string(found.worktree.join("nested/.git/HEAD")).expect("its own history is where it was"),
        "ref: refs/heads/inner\n"
    );
    assert!(
        !std::fs::read_dir(found.common.join(SAVED_DIRECTORY))
            .expect("the saved copies")
            .flatten()
            .any(|stamp| { return stamp.path().join("nested").exists(); }),
        "a put-back left a copy of the repository below the saved copies"
    );
    remove(found.root.as_path());
}

/// What:
///  A repository whose move the system refuses stops the save unchanged,
///  from a directory the command may not write,
///  and the reason is the system's own.
/// Why:
///  This is the shape a person meets on a real worktree:
///  a directory that can be read but not written to.
#[test]
fn a_move_the_system_refuses_changes_nothing() {
    use std::os::unix::fs::PermissionsExt;
    let found: Places = places("discarded-nested-repository-refused-write");
    std::fs::create_dir_all(found.worktree.join("a/.git")).expect("the first repository");
    std::fs::write(found.worktree.join("a/untracked.txt"), b"the person work\n").expect("its work");
    std::fs::create_dir_all(found.worktree.join("container/b/.git")).expect("the second repository");
    std::fs::write(found.worktree.join("container/b/untracked.txt"), b"more of the person work\n").expect("its work");
    std::fs::set_permissions(found.worktree.join("container"), std::fs::Permissions::from_mode(0o500))
        .expect("lock the container");
    let stopped = set_nested_repositories_aside(
        found.common.as_path(),
        found.worktree.as_path(),
        &[b"a".to_vec(), b"container/b".to_vec()],
        5_000,
    )
    .expect_err("the second move fails");
    std::fs::set_permissions(found.worktree.join("container"), std::fs::Permissions::from_mode(0o700))
        .expect("free the container");
    assert_eq!(stopped.failure.subject, "container/b");
    assert!(stopped.still_moved.is_empty(), "{:?}", stopped.still_moved);
    assert_eq!(
        std::fs::read_to_string(found.worktree.join("a/untracked.txt")).expect("the first repository is where it was"),
        "the person work\n"
    );
    assert_eq!(
        std::fs::read_to_string(found.worktree.join("container/b/untracked.txt")).expect("the second is where it was"),
        "more of the person work\n"
    );
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
