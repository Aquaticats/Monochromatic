//! What:
//!  The words a person reads when the object database cannot be trusted:
//!  a shallow repository,
//!  a replace ref,
//!  grafts,
//!  an alternates line in a quoting this reader does not follow,
//!  and a pack index of a version it does not read.
//! Why:
//!   A removal that refuses over one of these hands its reason to the person,
//!  so the reason names which store is in the way and what makes it plain again,
//!  and these controls hold it to that.

use super::*;
use crate::test_support::{fixture, git, remove, repository};
use std::path::PathBuf;

/// The reason `ObjectDatabase::open_or_reason` answers for the repository at `git_dir`.
fn refusal_of(git_dir: &std::path::Path) -> String {
    return match ObjectDatabase::open_or_reason(git_dir, None, ObjectFormat::Sha1) {
        Ok(_) => panic!("the store is opened where the control expects a refusal"),
        Err(reason) => reason,
    };
}

/// A repository whose `shallow` file marks its own history as cut short.
#[test]
fn a_shallow_repository_names_its_shallow_file_and_the_command_that_completes_it() {
    let root: PathBuf = fixture("repo-read-refusal-shallow");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let head = String::from_utf8(crate::test_support::git_output(repo.as_path(), &["rev-parse", "HEAD"]).stdout).expect("UTF-8");
    std::fs::write(repo.join(".git/shallow"), head.as_str()).expect("shallow");
    let reason: String = refusal_of(repo.join(".git").as_path());
    assert!(reason.contains("shallow"), "the reason names the shallow file: {reason}");
    assert!(reason.contains("git fetch --unshallow"), "the reason names the command that completes the repository: {reason}");
    remove(root.as_path());
}

/// A repository holding a replace ref,
/// which makes a name mean the object that ref names.
#[test]
fn a_replace_ref_names_the_ref_and_the_command_that_drops_it() {
    let root: PathBuf = fixture("repo-read-refusal-replace");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=second"]);
    let first = String::from_utf8(crate::test_support::git_output(repo.as_path(), &["rev-parse", "HEAD~1"]).stdout).expect("UTF-8");
    let second = String::from_utf8(crate::test_support::git_output(repo.as_path(), &["rev-parse", "HEAD"]).stdout).expect("UTF-8");
    git(repo.as_path(), &["replace", first.trim(), second.trim()]);
    let reason: String = refusal_of(repo.join(".git").as_path());
    assert!(reason.contains("refs/replace"), "the reason names the replace ref it found: {reason}");
    assert!(reason.contains("git replace --delete"), "the reason names the command that drops it: {reason}");
    remove(root.as_path());
}

/// A repository holding grafts.
#[test]
fn grafts_name_the_graft_file_and_what_replaces_them() {
    let root: PathBuf = fixture("repo-read-refusal-grafts");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::create_dir_all(repo.join(".git/info")).expect("info");
    std::fs::write(repo.join(".git/info/grafts"), "").expect("grafts");
    let reason: String = refusal_of(repo.join(".git").as_path());
    assert!(reason.contains("info/grafts"), "the reason names the graft file: {reason}");
    assert!(reason.contains("git filter-repo"), "the reason names what replaces a graft: {reason}");
    remove(root.as_path());
}

/// An alternates list naming a directory in a quoting this reader does not follow.
#[test]
fn an_alternates_line_in_a_quoting_this_reader_does_not_follow_names_the_file() {
    let root: PathBuf = fixture("repo-read-refusal-alternates-quoted");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::create_dir_all(repo.join(".git/objects/info")).expect("info");
    std::fs::write(repo.join(".git/objects/info/alternates"), "\"/nowhere/objects\"\n").expect("alternates");
    let reason: String = refusal_of(repo.join(".git").as_path());
    assert!(reason.contains("alternates"), "the reason names the alternates list: {reason}");
    assert!(reason.contains("quoting"), "the reason names what stops it: {reason}");
    assert!(reason.contains("write the path plainly"), "the reason says what makes it readable: {reason}");
    remove(root.as_path());
}

/// A pack index of a version this reader does not know,
/// which Git reads only because it wrote it.
#[test]
fn a_pack_index_of_an_unknown_version_names_the_index_and_the_command_that_rewrites_it() {
    let root: PathBuf = fixture("repo-read-refusal-index-version");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["repack", "-a", "-d", "-f", "--quiet"]);
    let index: PathBuf = std::fs::read_dir(repo.join(".git/objects/pack"))
        .expect("pack directory")
        .flatten()
        .map(|entry| { return entry.path(); })
        .find(|path| { return path.extension().is_some_and(|extension| { return extension == "idx"; }); })
        .expect("an index");
    let mut bytes: Vec<u8> = std::fs::read(&index).expect("the index's bytes");
    bytes[7] = 9;
    // Git writes its own files read-only.
    std::fs::set_permissions(&index, std::os::unix::fs::PermissionsExt::from_mode(0o644)).expect("writable");
    std::fs::write(&index, bytes).expect("the damaged index");
    let reason: String = refusal_of(repo.join(".git").as_path());
    assert!(reason.contains(index.to_str().expect("UTF-8")), "the reason names the index: {reason}");
    assert!(reason.contains("git index-pack"), "the reason names the command that writes the index again: {reason}");
    remove(root.as_path());
}
