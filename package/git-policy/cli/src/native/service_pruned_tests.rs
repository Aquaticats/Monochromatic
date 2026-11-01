//! Controls for recognizing content that was removed before a commit landed.

use super::*;
use crate::service_protocol::{ALLOWED_MODES, Change};
use crate::test_support::{fixture, git_output, remove};
use std::path::PathBuf;

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
