//! Controls for the text half of subsumption on real blobs:
//!  strict reverse application, one-sided extension, rejection of two-sided edits,
//!  binary exclusion, and an addition on both sides against the empty blob.

use super::*;
use crate::test_support::{fixture, git_context, git_text, remove, repository};
use std::path::PathBuf;

/// Writes `bytes` as a blob in the repository and returns a regular-file entry for it.
fn blob(repo: &Path, scratch: &Path, name: &str, bytes: &[u8]) -> Option<TreeEntry> {
    let file: PathBuf = scratch.join(name);
    std::fs::write(&file, bytes).expect("blob file");
    let oid: String = git_text(repo, &["hash-object", "-w", file.to_string_lossy().as_ref()]).trim().to_string();
    return Some(TreeEntry {
        mode: String::from("100644"),
        oid,
    });
}

/// A shared path whose three sides are the given bytes, or absent when `None`.
fn shared(repo: &Path, scratch: &Path, sides: [Option<&[u8]>; 3]) -> SharedPath {
    let entry = |index: usize, name: &str| -> Option<TreeEntry> {
        return sides[index].and_then(|bytes| { return blob(repo, scratch, name, bytes); });
    };
    return SharedPath {
        path: String::from("p.txt"),
        base: entry(0, "base"),
        landed: entry(1, "landed"),
        prepared: entry(2, "prepared"),
    };
}

/// Runs the text check over one candidate and reports whether it is subsumed.
fn subsumed(name: &str, base: Option<&[u8]>, landed: &[u8], prepared: &[u8]) -> bool {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let scratch: PathBuf = root.join("scratch");
    std::fs::create_dir(&scratch).expect("scratch");
    let candidate: SharedPath = shared(repo.as_path(), scratch.as_path(), [base, Some(landed), Some(prepared)]);
    let result: Vec<String> = subsumed_text_paths(
        &git_context(),
        repo.join(".git").as_path(),
        repo.as_path(),
        repo.join(".git").join("objects").as_path(),
        &[candidate],
    )
    .expect("text check");
    remove(root.as_path());
    return !result.is_empty();
}

/// A landed edit already in the prepared bytes is subsumed by strict reverse application.
#[test]
fn strict_reverse_application_subsumes_the_landed_edit() {
    assert!(subsumed("strict_reverse_application_subsumes_the_landed_edit", Some(b"a\nb\nc\n"), b"a\nX\nc\n", b"a\nX\nc\n"));
}

/// An own edit next to the landed edit is subsumed by the one-sided extension, which strict application rejects.
#[test]
fn an_adjacent_own_edit_is_subsumed_by_the_one_sided_extension() {
    assert!(subsumed("an_adjacent_own_edit_is_subsumed_by_the_one_sided_extension", Some(b"a\nb\nc\nd\n"), b"a\nB\nc\nd\n", b"a\nB\nC\nd\n"));
}

/// Own edits on both sides of the landed edit are not subsumed.
#[test]
fn own_edits_on_both_sides_are_not_subsumed() {
    assert!(!subsumed("own_edits_on_both_sides_are_not_subsumed", Some(b"a\nb\nc\nd\n"), b"a\nB\nc\nd\n", b"a\nX\nB\nC\nd\n"));
}

/// A prepared file that drops the landed edit is not subsumed.
#[test]
fn a_prepared_file_without_the_landed_edit_is_not_subsumed() {
    assert!(!subsumed("a_prepared_file_without_the_landed_edit_is_not_subsumed", Some(b"a\nb\nc\n"), b"a\nX\nc\n", b"a\nb\nc\n"));
}

/// An addition on both sides, read against the empty blob, is subsumed when the prepared file keeps every landed line in order.
#[test]
fn an_addition_on_both_sides_is_subsumed_against_the_empty_blob() {
    assert!(subsumed("an_addition_on_both_sides_is_subsumed_against_the_empty_blob", None, b"a\nb\n", b"a\nX\nb\n"));
    assert!(!subsumed("an_addition_on_both_sides_is_subsumed_against_the_empty_blob", None, b"a\nb\n", b"a\nX\n"));
}

/// A binary blob is never a text candidate, even when the bytes are identical.
#[test]
fn a_binary_blob_is_never_subsumed_by_the_text_check() {
    assert!(!subsumed("a_binary_blob_is_never_subsumed_by_the_text_check", Some(b"x\0a\n"), b"x\0B\n", b"x\0B\n"));
}

/// The binary rule looks at the first 8000 bytes only, and an empty input is never binary.
#[test]
fn binary_detection_probes_the_first_bytes() {
    assert!(is_binary(b"a\0"));
    assert!(!is_binary(b""));
    let mut late: Vec<u8> = vec![b'a'; BINARY_PROBE_BYTES];
    late.push(0);
    assert!(!is_binary(&late));
}

/// No candidates means no Git process and no decisions.
#[test]
fn no_candidates_decide_nothing() {
    let root: PathBuf = fixture("replay-subsumption-empty");
    let repo: PathBuf = repository(root.as_path(), "repo");
    assert!(subsumed_text_paths(
        &git_context(),
        repo.join(".git").as_path(),
        repo.as_path(),
        repo.join(".git").join("objects").as_path(),
        &[],
    )
    .expect("empty")
    .is_empty());
    remove(root.as_path());
}
