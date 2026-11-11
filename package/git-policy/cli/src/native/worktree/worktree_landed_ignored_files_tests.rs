//! What:
//!  Controls for the ignored entries of a worktree that nothing else in the repository holds.
//! Why:
//!   The sweep keeps a worktree only on the proof this question gives,
//!  so each answer needs a control that would catch it going:
//! a file the source lacks or holds with other bytes is named,
//! the same bytes are not,
//! and an ignored directory is never read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(ignoredEntriesAloneHere(git, worktree, admin, main).shown).toEqual(['.env']);
//! ```

use super::{AloneHere, ignored_entries_alone_here};
use crate::test_support::{age, fixture, git, git_context, repository};
use std::path::{Path, PathBuf};

/// What:
///  A repository whose worktree ignores `.env` and holds one worktree.
/// Why:
///   Every control below asks the same question about the same shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { main, worktree, admin } = aRepositoryWithAIgnoredWorktree(name, '.env\\n');
/// ```
struct Ignored {
    /// The checkout the merge runs in.
    main: PathBuf,
    /// The linked worktree.
    worktree: PathBuf,
    /// Its administrative directory.
    admin: PathBuf,
}

fn a_repository_with_an_ignored_worktree(name: &str, rules: &str) -> Ignored {
    let root: PathBuf = fixture(name);
    let main: PathBuf = repository(root.as_path(), "main");
    let tracked: PathBuf = main.join("README.md");
    std::fs::write(tracked.as_path(), b"# fixture\n").expect("a tracked file");
    let ignored: PathBuf = main.join(".gitignore");
    std::fs::write(ignored.as_path(), rules.as_bytes()).expect("the ignore rules");
    age(tracked.as_path());
    age(ignored.as_path());
    git(main.as_path(), &["add", "--", ".gitignore", "README.md"]);
    git(main.as_path(), &["commit", "--quiet", "--message=a tracked file"]);
    let worktree: PathBuf = root.join("worktree");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "feature", worktree.to_str().expect("a UTF-8 fixture path")]);
    let admin: PathBuf = the_worktrees_admin(main.join(".git").as_path());
    return Ignored { main, worktree, admin };
}

/// The one administrative directory the fixture's common Git directory holds.
fn the_worktrees_admin(common: &Path) -> PathBuf {
    let listed: Vec<PathBuf> = std::fs::read_dir(common.join("worktrees")).expect("the worktrees folder").flatten().map(|entry| entry.path()).collect();
    assert_eq!(listed.len(), 1, "the fixture registers one worktree: {listed:?}");
    return listed[0].clone();
}

#[test]
fn an_ignored_file_the_repository_does_not_hold_is_named() {
    let ignored: Ignored = a_repository_with_an_ignored_worktree("ignored-alone-own", ".env\n");
    std::fs::write(ignored.worktree.join(".env").as_path(), b"PASSWORD=the person's own\n").expect("a file the person wrote");
    let alone: AloneHere = ignored_entries_alone_here(&git_context(), ignored.worktree.as_path(), ignored.admin.as_path(), ignored.main.as_path()).expect("Git lists the ignored files");
    assert!(!alone.is_empty(), "the file is alone here: {alone:?}");
    assert_eq!(alone.count, 1, "{alone:?}");
    assert_eq!(alone.shown, vec![String::from(".env")], "{alone:?}");
    assert_eq!(alone.list(), ".env");
}

#[test]
fn an_ignored_file_the_repository_holds_with_the_same_bytes_is_not_named() {
    let ignored: Ignored = a_repository_with_an_ignored_worktree("ignored-alone-shared", ".env\n");
    std::fs::write(ignored.main.join(".env").as_path(), b"A=1\n").expect("the repository's own file");
    std::fs::write(ignored.worktree.join(".env").as_path(), b"A=1\n").expect("the copy the add carries");
    let alone: AloneHere = ignored_entries_alone_here(&git_context(), ignored.worktree.as_path(), ignored.admin.as_path(), ignored.main.as_path()).expect("Git lists the ignored files");
    assert!(alone.is_empty(), "the bytes are the repository's own: {alone:?}");
}

#[test]
fn an_ignored_file_the_repository_holds_with_other_bytes_is_named() {
    let ignored: Ignored = a_repository_with_an_ignored_worktree("ignored-alone-other-bytes", ".env\n");
    std::fs::write(ignored.main.join(".env").as_path(), b"PORT=3000\n").expect("the repository's own file");
    std::fs::write(ignored.worktree.join(".env").as_path(), b"PORT=3001\n").expect("the person's own port");
    let alone: AloneHere = ignored_entries_alone_here(&git_context(), ignored.worktree.as_path(), ignored.admin.as_path(), ignored.main.as_path()).expect("Git lists the ignored files");
    assert_eq!(alone.count, 1, "the edited file is alone here: {alone:?}");
    assert_eq!(alone.shown, vec![String::from(".env")], "{alone:?}");
}

#[test]
fn an_ignored_directory_is_never_read() {
    let ignored: Ignored = a_repository_with_an_ignored_worktree("ignored-alone-build", "build/\n");
    std::fs::create_dir(ignored.worktree.join("build").as_path()).expect("a build directory of the worktree's own");
    std::fs::write(ignored.worktree.join("build").join("output.o").as_path(), b"build output\n").expect("a file of that directory");
    let alone: AloneHere = ignored_entries_alone_here(&git_context(), ignored.worktree.as_path(), ignored.admin.as_path(), ignored.main.as_path()).expect("Git lists the ignored files");
    assert!(alone.is_empty(), "a directory is the build state the sweep gives back: {alone:?}");
}

#[test]
fn a_list_longer_than_the_shown_names_counts_the_rest() {
    let three: AloneHere = AloneHere { shown: vec![String::from("a"), String::from("b"), String::from("c")], count: 3 };
    assert_eq!(three.list(), "a, b, c");
    let four: AloneHere = AloneHere { shown: vec![String::from("a"), String::from("b"), String::from("c")], count: 4 };
    assert_eq!(four.list(), "a, b, c, 1 more");
    let five: AloneHere = AloneHere { shown: vec![String::from("a"), String::from("b"), String::from("c")], count: 5 };
    assert_eq!(five.list(), "a, b, c, 2 more");
    assert!(AloneHere::default().is_empty(), "nothing named is nothing alone");
}
