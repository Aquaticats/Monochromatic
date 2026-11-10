//! Controls for the fill the service runs ahead of a commit:
//!  the facts it reads are the ones the invocation's capture reads,
//! and the member it makes is one a commit then takes.

use super::*;
use crate::capture_invocation::{capture_invocation_base, capture_invocation_layout};
use crate::shadow_pool::{take, pool_root};
use crate::test_support::{fixture, git, git_context, remove, repository};
use std::path::{Path, PathBuf};

/// A repository with one commit on `main` and one tracked file.
fn repository_of(root: &Path) -> PathBuf {
    let repo: PathBuf = repository(root, "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    git(repo.as_path(), &["add", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=a"]);
    return repo;
}

/// The facts a fill reads are the facts the invocation's own capture reads,
///  field by field.
/// Why:
///  A member is refused unless its config and alternates are what the claiming invocation would write,
///  and this is the control that keeps the service's reading of a repository equal to a commit's.
#[test]
fn the_facts_a_fill_reads_are_the_facts_a_capture_reads() {
    let root: PathBuf = fixture("service-pool-fill-facts");
    let repo: PathBuf = repository_of(root.as_path());
    let context: GitContext = git_context();
    let layout = capture_invocation_layout(&context, repo.as_path(), false).expect("layout");
    let expected: ShadowTemplate = ShadowTemplate::of(&context, &layout).expect("the client's facts");
    let filled: ShadowTemplate = template_of(
        &context,
        repo.join(".git").as_path(),
        repo.join(".git").as_path(),
        repo.as_path(),
    )
    .expect("the service's facts");
    assert_eq!(filled.common_dir, expected.common_dir, "the common directory");
    assert_eq!(filled.git_dir, expected.git_dir, "the worktree's Git directory");
    assert_eq!(filled.repository_root, repo, "the worktree root");
    assert_eq!(filled.object_directory, expected.object_directory, "the object directory");
    assert_eq!(filled.object_format, expected.object_format, "the object format");
    assert_eq!(filled.ref_format, expected.ref_format, "the ref backend");
    assert_eq!(filled.format, expected.format, "the format facts");
    remove(root.as_path());
}

/// A fill from a request's facts makes a member a commit takes,
///  and the commit's own facts accept it.
#[test]
fn a_fill_from_a_request_makes_a_member_a_commit_takes() {
    let root: PathBuf = fixture("service-pool-fill-takes");
    let repo: PathBuf = repository_of(root.as_path());
    let context: GitContext = git_context();
    let made: Option<PathBuf> = fill_one(
        &context,
        repo.join(".git").as_path(),
        repo.join(".git").as_path(),
        repo.as_path(),
    )
    .expect("filled");
    assert!(made.is_some(), "a member was made");
    let layout = capture_invocation_layout(&context, repo.as_path(), false).expect("layout");
    let capture = capture_invocation_base(&context, repo.as_path(), layout).expect("capture");
    let template: ShadowTemplate = ShadowTemplate::of(&context, &capture.layout).expect("facts");
    let destination: PathBuf = root.join("shadow");
    assert!(take(&template, destination.as_path()), "the commit takes the member");
    assert!(members(pool_root(&template).as_path()).is_empty(), "and the pool is empty");
    remove(root.as_path());
}

/// A pool that already holds what a fill would make is left alone.
#[test]
fn a_full_pool_is_not_filled_again() {
    let root: PathBuf = fixture("service-pool-fill-full");
    let repo: PathBuf = repository_of(root.as_path());
    let context: GitContext = git_context();
    let common: PathBuf = repo.join(".git");
    let mut made: usize = 0;
    for _round in 0..POOL_LIMIT + 1 {
        if fill_one(&context, common.as_path(), common.as_path(), repo.as_path()).expect("filled").is_some() {
            made += 1;
        }
    }
    assert_eq!(made, POOL_LIMIT, "a fill stops at the pool's limit");
    remove(root.as_path());
}
/// What:
///  A repository at `directory`, with one commit behind it.
/// Why:
///   A fill asks Git the same questions a capture does,
///  and a capture needs a repository with a `HEAD`.
fn initialize(directory: PathBuf) -> PathBuf {
    std::fs::create_dir(directory.as_path()).expect("repository directory");
    git(directory.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    git(directory.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    return directory;
}

/// The facts the service's fill reads in `repo`.
fn template_of_fixture(repo: &Path) -> ShadowTemplate {
    return template_of(&git_context(), repo.join(".git").as_path(), repo.join(".git").as_path(), repo).expect("the service's facts");
}

/// The facts the invocation's own capture reads in `repo`.
fn capture_layout(repo: &Path) -> crate::capture_invocation::InvocationLayout {
    return capture_invocation_layout(&git_context(), repo, false).expect("layout");
}

/// What:
///  The facts a fill reads of a repository whose directory name is not UTF-8 are the facts its capture reads.
/// Why:
///   The fill decoded Git's answer as text,
///  so the object directory it named held U+FFFD where the name held a byte,
/// and the member's alternates named a store that does not exist.
#[test]
fn the_facts_of_a_repository_whose_name_is_not_text_are_the_captures() {
    use std::os::unix::ffi::OsStringExt;
    let root: PathBuf = fixture("service-pool-fill-not-utf8");
    let repo: PathBuf = initialize(root.join(OsString::from_vec(b"rep\xefo".to_vec())));
    let filled: ShadowTemplate = template_of_fixture(repo.as_path());
    assert_eq!(
        filled.object_directory,
        capture_layout(repo.as_path()).object_directory,
        "the object directory is the bytes Git printed"
    );
    remove(root.as_path());
}

/// What:
///  The facts a fill reads of a repository whose directory name holds a line feed are the facts its capture reads.
/// Why:
///   The fill split Git's answer at every line feed,
///  and such a name puts one inside the first answer,
/// so the object directory it read was the part before the name's own line feed.
#[test]
fn the_facts_of_a_repository_whose_name_holds_a_line_feed_are_the_captures() {
    let root: PathBuf = fixture("service-pool-fill-line-feed");
    let repo: PathBuf = initialize(root.join("a\nb"));
    let filled: ShadowTemplate = template_of_fixture(repo.as_path());
    assert_eq!(
        filled.object_directory,
        capture_layout(repo.as_path()).object_directory,
        "one answer holds the whole pathname"
    );
    remove(root.as_path());
}
