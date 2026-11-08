//! Controls for the registration lookup of a worktree removal:
//! Git's match by last components,
//! the scope of the
//!  registration to the invoking repository,
//! and the pointers that must name each other.

use super::*;
use crate::test_support::{fixture, git, remove, repository};

/// Git's suffix match starts at a directory boundary,
/// ignores nothing but ASCII case,
/// and needs a non-empty name.
#[test]
fn the_last_components_match_at_a_directory_boundary() {
    for (candidate, operand, expected) in [
        ("/a/b/topic", "topic", true),
        ("/a/b/topic", "b/topic", true),
        ("/a/b/topic", "/a/b/topic", true),
        ("/a/b/mytopic", "topic", false),
        ("/a/b/topic", "Topic", true),
        ("/a/b/topic", "", false),
        ("/b/topic", "/a/b/topic", false),
        ("/a/b/topic", "topic/", false),
    ] {
        assert_eq!(ends_with_name(candidate.as_bytes(), operand.as_bytes()), expected, "{candidate} against {operand}");
    }
}

/// A linked worktree is found in the repository that registered it and in no other;
/// a pointer pair that disagrees is not
///  a registration.
#[test]
fn a_registration_belongs_to_the_invoking_repository_and_the_pointers_agree() {
    let root: PathBuf = fixture("registry-scope");
    let first: PathBuf = repository(root.as_path(), "first");
    let second: PathBuf = repository(root.as_path(), "second");
    git(first.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../wt"]);
    let worktree: PathBuf = std::fs::canonicalize(root.join("wt")).expect("canonical");
    let first_common: PathBuf = std::fs::canonicalize(first.join(".git")).expect("common");
    let second_common: PathBuf = std::fs::canonicalize(second.join(".git")).expect("common");
    let found: Registration = registration_of(first_common.as_path(), worktree.as_path(), worktree.as_os_str()).expect("registered");
    assert_eq!(found.id, OsString::from("wt"));
    assert_eq!(found.admin, first_common.join("worktrees/wt"));
    let none: String = registration_of(second_common.as_path(), worktree.as_path(), worktree.as_os_str()).expect_err("a repository with no linked worktree");
    assert!(none.contains("no linked worktree"), "{none}");
    git(second.as_path(), &["worktree", "add", "--quiet", "-b", "other", "../other"]);
    let refused: String = registration_of(second_common.as_path(), worktree.as_path(), worktree.as_os_str()).expect_err("another repository's worktree");
    assert!(refused.contains("not one of the registrations"), "{refused}");
    // The main checkout has no registration in its own repository.
    let main: String = registration_of(first_common.as_path(), first.as_path(), first.as_os_str()).expect_err("the main checkout");
    assert!(main.contains(".git"), "{main}");
    // A `gitdir` file that records another place disagrees with the worktree's `.git` file.
    std::fs::write(first_common.join("worktrees/wt/gitdir"), b"/elsewhere/.git\n").expect("overwrite");
    let disagreeing: String = registration_of(first_common.as_path(), worktree.as_path(), worktree.as_os_str()).expect_err("disagreeing pointers");
    assert!(disagreeing.contains("records /elsewhere/.git"), "{disagreeing}");
    remove(root.as_path());
}

/// A registration that spells the worktree through a symbolic link belongs to the same worktree,
/// so a removal named either way is served;
/// a registration that records another place is still refused.
#[test]
fn a_registration_spelled_through_a_link_belongs_to_the_same_worktree() {
    let root: PathBuf = fixture("registry-link");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../wt"]);
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common");
    // The worktree moved behind a link: a link is left at the old name,
    // so the registration still spells the old name while naming the same directory.
    let moved: PathBuf = root.join("moved");
    std::fs::rename(root.join("wt"), &moved).expect("rename the worktree");
    std::os::unix::fs::symlink(&moved, root.join("wt")).expect("leave a link at the old name");
    let typed: PathBuf = root.join("wt");
    let found: Registration = registration_of(common.as_path(), moved.as_path(), typed.as_os_str()).expect("the two spellings are one worktree");
    assert_eq!(found.id, OsString::from("wt"));
    assert_eq!(found.admin, common.join("worktrees/wt"));
    registration_of(common.as_path(), moved.as_path(), OsStr::new("moved")).expect("the name matches only the target");
    // A record that names another place is still refused.
    std::fs::write(common.join("worktrees/wt/gitdir"), b"/elsewhere/.git\n").expect("overwrite");
    let elsewhere: String = registration_of(common.as_path(), moved.as_path(), typed.as_os_str()).expect_err("another place");
    assert!(elsewhere.contains("records /elsewhere/.git"), "{elsewhere}");
    remove(root.as_path());
}

/// A name that Git would match to a different registered worktree is refused with both paths named;
/// a name that matches
///  only the target is accepted.
#[test]
fn a_name_that_matches_another_registered_worktree_is_refused() {
    let root: PathBuf = fixture("registry-suffix");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "one", "../x/topic"]);
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "two", "../y/other"]);
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common");
    let other: PathBuf = std::fs::canonicalize(root.join("y/other")).expect("canonical");
    let topic: PathBuf = std::fs::canonicalize(root.join("x/topic")).expect("canonical");
    let refused: String = registration_of(common.as_path(), other.as_path(), OsStr::new("topic")).expect_err("Git picks x/topic for this name");
    assert!(refused.contains(topic.to_str().expect("UTF-8")) && refused.contains(other.to_str().expect("UTF-8")), "{refused}");
    registration_of(common.as_path(), topic.as_path(), OsStr::new("topic")).expect("the name picks the target itself");
    registration_of(common.as_path(), other.as_path(), other.as_os_str()).expect("a full path matches only itself");
    remove(root.as_path());
}
