//! Controls for recognizing content that was removed before a commit landed.

use super::*;
use crate::service_protocol::{ALLOWED_MODES, Change};
use crate::test_support::{fixture, git_output, remove};
use std::path::{Path, PathBuf};

const NAME: &str = "0123456789abcdef0123456789abcdef01234567";

fn failure(code: EngineFailureCode, message: &str) -> PolicyEvent {
    return PolicyEvent::EngineFailure { code, message: message.to_string(), trigger: None, policy: None, path: None };
}

/// The failure a policy raises for a missing object is the removed-content signal; the same code with another cause, and
///  other codes, are not.
#[test]
fn only_a_missing_object_is_removed_content() {
    let missing: String = format!("{OBJECT_MISSING_PREFIX}{NAME}. Run `git fsck` to check the object store.");
    assert!(content_was_removed(&[failure(EngineFailureCode::ContentUnavailable, missing.as_str())]));
    assert!(!content_was_removed(&[failure(EngineFailureCode::ContentUnavailable, "cli-git could not start Git")]));
    assert!(!content_was_removed(&[failure(EngineFailureCode::PolicyIncomplete, missing.as_str())]));
    assert!(!content_was_removed(&[]));
}

fn request(blob: &str) -> CommitRequest {
    return CommitRequest {
        id: String::from("x"),
        repository: PathBuf::from("/nowhere"),
        common_dir: PathBuf::from("/nowhere/.git"),
        git_dir: PathBuf::from("/nowhere/.git"),
        reference: String::from("refs/heads/main"),
        base: NAME.to_string(),
        changes: vec![Change { path: String::from("a.txt"), mode: ALLOWED_MODES[0], blob: Some(blob.to_string()) }],
        message: String::from("m\n"),
        author: None,
        committer: String::from("T <t@example.invalid> 1793535889 +0000"),
        amend: false,
        tree: None,
        commit: None,
        cleanup: None,
        index: PathBuf::from("/nowhere/.git/index"),
        captured: 0,
    };
}

/// An object the request names is present while its loose file is there and missing once the file is gone.
#[test]
fn a_named_object_is_missing_once_its_loose_file_is_gone() {
    let root: PathBuf = fixture("pruned-objects");
    git_output(root.as_path(), &["init", "--quiet"]);
    std::fs::write(root.join("a.txt"), "content\n").expect("file");
    let written = git_output(root.as_path(), &["hash-object", "-w", "a.txt"]);
    assert!(written.status.success());
    let name: String = String::from_utf8_lossy(&written.stdout).trim().to_string();
    let common: PathBuf = root.join(".git");
    let objects: PathBuf = common.join("objects");
    assert!(!prepared_content_missing(common.as_path(), objects.as_path(), "sha1", &request(name.as_str())));
    std::fs::remove_file(objects.join(&name[..2]).join(&name[2..])).expect("the object is loose");
    assert!(prepared_content_missing(common.as_path(), objects.as_path(), "sha1", &request(name.as_str())));
    remove(root.as_path());
}

/// Each numbered phase file answers one occurrence: the marker counts how often the phase was reached.
#[test]
fn the_service_phases_are_numbered_by_occurrence() {
    let before: u64 = STAGED_REACHED.load(Ordering::SeqCst);
    // Unarmed, reaching a phase does nothing but count.
    reach_service_phase(Phase::ServiceStaged).expect("unarmed");
    reach_service_phase(Phase::ServiceStaged).expect("unarmed");
    assert_eq!(STAGED_REACHED.load(Ordering::SeqCst), before + 2);
}

/// A fixture repository holding `d/a.txt`, its trees, and an amendment-shaped request that names only the root tree.
struct TreeFixture {
    root: PathBuf,
    blob: String,
    subtree: String,
    request: CommitRequest,
}

fn tree_fixture(name: &str) -> TreeFixture {
    let root: PathBuf = fixture(name);
    git_output(root.as_path(), &["init", "--quiet"]);
    std::fs::create_dir(root.join("d")).expect("directory");
    std::fs::write(root.join("d").join("a.txt"), "content\n").expect("file");
    assert!(git_output(root.as_path(), &["add", "d/a.txt"]).status.success());
    let text = |arguments: &[&str]| -> String { return String::from_utf8_lossy(&git_output(root.as_path(), arguments).stdout).trim().to_string(); };
    let tree: String = text(&["write-tree"]);
    let blob: String = text(&["rev-parse", ":d/a.txt"]);
    let subtree: String = text(&["rev-parse", format!("{tree}:d").as_str()]);
    let mut request: CommitRequest = request(blob.as_str());
    request.changes = Vec::new();
    request.amend = true;
    request.tree = Some(tree);
    return TreeFixture { root, blob, subtree, request };
}

fn remove_loose(root: &Path, name: &str) {
    std::fs::remove_file(root.join(".git").join("objects").join(&name[..2]).join(&name[2..])).expect("the object is loose");
}

/// An amendment that embeds a whole tree is intact only while every blob and tree below it is: a blob removed from under a
///  surviving tree is removed content, found before the branch moves.
#[test]
fn a_blob_removed_from_under_an_embedded_tree_is_removed_content() {
    let fixed = tree_fixture("pruned-tree-blob");
    let common: PathBuf = fixed.root.join(".git");
    assert!(!batch_content_missing(common.as_path(), &[&fixed.request]));
    remove_loose(fixed.root.as_path(), fixed.blob.as_str());
    assert!(batch_content_missing(common.as_path(), &[&fixed.request]));
    assert!(prepared_content_missing(common.as_path(), common.join("objects").as_path(), "sha1", &fixed.request));
    remove(fixed.root.as_path());
}

/// A subtree removed from under the embedded tree is removed content too.
#[test]
fn a_subtree_removed_from_under_an_embedded_tree_is_removed_content() {
    let fixed = tree_fixture("pruned-tree-subtree");
    let common: PathBuf = fixed.root.join(".git");
    remove_loose(fixed.root.as_path(), fixed.subtree.as_str());
    assert!(batch_content_missing(common.as_path(), &[&fixed.request]));
    assert!(prepared_content_missing(common.as_path(), common.join("objects").as_path(), "sha1", &fixed.request));
    remove(fixed.root.as_path());
}

/// A submodule commit below the embedded tree lives in another repository, so its absence here is not removed content.
#[test]
fn a_submodule_entry_below_an_embedded_tree_is_not_removed_content() {
    let fixed = tree_fixture("pruned-tree-gitlink");
    let common: PathBuf = fixed.root.join(".git");
    let added = git_output(
        fixed.root.as_path(),
        &["update-index", "--add", "--cacheinfo", "160000,0123456789abcdef0123456789abcdef01234567,sub"],
    );
    assert!(added.status.success());
    let written = git_output(fixed.root.as_path(), &["write-tree"]);
    let mut request: CommitRequest = fixed.request.clone();
    request.tree = Some(String::from_utf8_lossy(&written.stdout).trim().to_string());
    assert!(!batch_content_missing(common.as_path(), &[&request]));
    assert!(!prepared_content_missing(common.as_path(), common.join("objects").as_path(), "sha1", &request));
    remove(fixed.root.as_path());
}
