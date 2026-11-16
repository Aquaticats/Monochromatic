//! What:
//!  The cleanliness proof over a store whose objects are packed:
//!  a repository `git gc` emptied of loose objects is proven clean,
//!  and a store that makes an object name mean another object is refused in the store's own words.
//! Why:
//!   A worktree's removal by subvolume destroy rests on this proof,
//!  and the proof reads the tree of `HEAD` through the in-process object reader,
//!  so a packed store must be read here and a store that cannot be trusted must refuse here,
//!  naming the store a person can act on.

use super::*;
use crate::test_support::{age, fixture, git, git_output, git_context, remove, repository};
use std::path::PathBuf;

/// A repository of two tracked files and an ignored directory,
/// all older than the index,
/// whose objects a `git gc` moved into a pack.
fn packed_repository(root: &Path) -> PathBuf {
    let repo: PathBuf = repository(root, "repo");
    std::fs::write(repo.join(".gitignore"), b"ignored/\n").expect("gitignore");
    std::fs::write(repo.join("a.txt"), b"alpha\n").expect("a");
    std::fs::create_dir(repo.join("dir")).expect("dir");
    std::fs::write(repo.join("dir/b.txt"), b"bravo\n").expect("b");
    for name in [".gitignore", "a.txt", "dir/b.txt"] {
        age(repo.join(name).as_path());
    }
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=files"]);
    git(repo.as_path(), &["gc", "--quiet", "--prune=now"]);
    return repo;
}

/// The cleanliness proof of the repository at `repo`.
fn check(repo: &Path) -> Result<CleanSource, CleanRefusal> {
    let git_dir: PathBuf = repo.join(".git");
    return check_clean(&git_context(), repo, git_dir.as_path(), git_dir.as_path());
}

/// What:
///  A repository whose objects all sit in a pack,
///  with files older than its index,
///  is proven clean.
/// Why:
///   The proof's own read of the tree of `HEAD` crosses the packed store:
///  a reader that refuses a pack would refuse every worktree of this repository,
///  and every removal that rests on this proof would fall back to Git or refuse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = packedRepository(root); expect(checkClean(repo).ok).toBe(true);
/// ```
#[test]
fn a_repository_whose_objects_are_packed_is_proven_clean() {
    let root: PathBuf = fixture("clean-packed-store");
    let repo: PathBuf = packed_repository(root.as_path());
    let counts = String::from_utf8(git_output(repo.as_path(), &["count-objects", "-v"]).stdout).expect("UTF-8");
    // Positive control: the fixture really holds no loose object, so every object the proof reads comes out of a pack.
    assert!(counts.lines().any(|line| { return line == "count: 0"; }), "the store holds no loose object: {counts}");
    let clean: CleanSource = check(repo.as_path()).expect("a packed store is read");
    assert_eq!(clean.tracked, 3, "the three tracked files are counted");
    remove(root.as_path());
}

/// A store whose replace ref makes an object name mean another object is refused,
/// in the store's own words.
#[test]
fn a_store_that_makes_a_name_mean_another_object_is_refused_in_its_own_words() {
    let root: PathBuf = fixture("clean-replaced-store");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=second"]);
    let first = String::from_utf8(git_output(repo.as_path(), &["rev-parse", "HEAD~1"]).stdout).expect("UTF-8");
    let second = String::from_utf8(git_output(repo.as_path(), &["rev-parse", "HEAD"]).stdout).expect("UTF-8");
    git(repo.as_path(), &["replace", first.trim(), second.trim()]);
    let refusal: CleanRefusal = check(repo.as_path()).expect_err("a replaced name proves nothing");
    assert_eq!(refusal.kind, CleanRefusalKind::Unreadable, "the refusal is the store's");
    assert!(refusal.reason.contains("refs/replace"), "the reason names the replace ref: {}", refusal.reason);
    assert!(refusal.reason.contains("git replace --delete"), "the reason names what drops it: {}", refusal.reason);
    remove(root.as_path());
}
