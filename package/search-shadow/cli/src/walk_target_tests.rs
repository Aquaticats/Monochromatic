//! What:
//! Unit tests of the walk-target classifier in `walk_target.rs`.
//! Why:
//! Every `grep` and `find` refusal rests on this one question,
//! so each answer needs a case on a real directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // walk-target.unit.test.ts
//! describe('classify', () => { ... });
//! ```

/// Imports the classifier and its phrases from this crate.
use crate::walk_target::{WalkTarget, classify, phrase};

/// Imports the file-system module (make fixture directories).
use std::fs;

/// What:
/// `#[test]` marks the next function as a test case.
/// Why:
/// The runner finds the tests by it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// it('a directory with .git is a repository root', () => { ... });
/// ```
#[test]
fn a_directory_that_contains_dot_git_is_a_repository_root() {
    // `scratch()` makes a directory that is deleted when the variable goes out of scope.
    let repository = crate::test_scratch::scratch();
    fs::create_dir(repository.path().join(".git")).expect("make .git");
    let home = crate::test_scratch::scratch();

    // `assert_eq!(a, b)` stops the test with a message when `a != b` (a macro, hence `!`).
    assert_eq!(classify(".", repository.path(), home.path()), WalkTarget::RepositoryRoot);
    assert_eq!(classify("./", repository.path(), home.path()), WalkTarget::RepositoryRoot);
    // An absolute spelling, from another directory, names the same repository.
    let elsewhere = crate::test_scratch::scratch();
    let absolute = repository.path().to_string_lossy().to_string();
    assert_eq!(classify(&absolute, elsewhere.path(), home.path()), WalkTarget::RepositoryRoot);
}

/// A plain subdirectory of a repository is bounded:
/// its walk stays inside it.
#[test]
fn a_subdirectory_of_a_repository_is_bounded() {
    let repository = crate::test_scratch::scratch();
    fs::create_dir(repository.path().join(".git")).expect("make .git");
    fs::create_dir(repository.path().join("package")).expect("make a subdirectory");
    let home = crate::test_scratch::scratch();

    assert_eq!(classify("package", repository.path(), home.path()), WalkTarget::Bounded);
}

/// The home directory and the filesystem root are wide whatever they contain.
#[test]
fn the_home_directory_and_the_filesystem_root_are_wide() {
    let home = crate::test_scratch::scratch();
    let work = crate::test_scratch::scratch();
    let home_text = home.path().to_string_lossy().to_string();

    assert_eq!(classify(&home_text, work.path(), home.path()), WalkTarget::Home);
    assert_eq!(classify("/", work.path(), home.path()), WalkTarget::FilesystemRoot);
}

/// A path that does not exist walks nothing,
/// so it is bounded.
#[test]
fn a_missing_path_is_bounded() {
    let work = crate::test_scratch::scratch();
    let home = crate::test_scratch::scratch();

    assert_eq!(classify("no/such/directory", work.path(), home.path()), WalkTarget::Bounded);
}

/// Each wide case has its own phrase for the refusal message.
#[test]
fn each_wide_case_has_a_phrase() {
    assert_eq!(phrase(&WalkTarget::RepositoryRoot), "a whole repository");
    assert_eq!(phrase(&WalkTarget::Home), "your whole home directory");
    assert_eq!(phrase(&WalkTarget::FilesystemRoot), "the whole filesystem");
    assert_eq!(phrase(&WalkTarget::Bounded), "a directory tree");
}
