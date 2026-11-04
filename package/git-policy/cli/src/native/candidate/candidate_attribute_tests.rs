//! What:
//!  Controls for the `text` attribute resolver:
//!  the parsing of `git check-attr -z` records and the failures of a lookup.
//! Why:
//!  Only the value `unset` preserves a file,
//!  and a reply that does not match the question must fail the lookup
//!  instead of defaulting a file to text or to binary.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(parseTextUnset(reply, asked, paths)).toEqual([Buffer.from('seed/case')]);
//! ```

/// Import the module under test.
use super::{AttributeSource, CandidateAttributes, parse_text_unset};
use crate::candidate_error::{CandidateError, CandidateFailure};
use crate::candidate_object::{ObjectId, parse_object_id};
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use crate::test_support::{REAL_GIT, fixture, git, remove, repository};
use std::path::{Path, PathBuf};

/// The names asked about, as owned byte lists.
fn asked(names: &[&str]) -> Vec<Vec<u8>> {
    return names.iter().map(|name| { return name.as_bytes().to_vec(); }).collect();
}

/// The failure cause of a parse that must fail.
fn failure_of(result: Result<Vec<Vec<u8>>, CandidateError>) -> CandidateFailure {
    return result.expect_err("a malformed reply").failure;
}

/// Only the exact value `unset` is reported,
///  in the order asked.
#[test]
fn only_an_unset_value_is_reported() {
    let reply: &[u8] = b"a\0text\0unset\0b\0text\0set\0c\0text\0unspecified\0d\0text\0auto\0e\0text\0unsetx\0f\0text\0unset\0";
    let names: Vec<Vec<u8>> = asked(&["a", "b", "c", "d", "e", "f"]);
    let paths: [&[u8]; 6] = [b"a", b"b", b"c", b"d", b"e", b"f"];
    assert_eq!(
        parse_text_unset(reply, names.as_slice(), &paths).expect("parsed"),
        vec![b"a".to_vec(), b"f".to_vec()]
    );
    // The reported name is the candidate's, without the top level that was asked.
    assert_eq!(
        parse_text_unset(
            b"/top/sub/x\0text\0unset\0",
            asked(&["/top/sub/x"]).as_slice(),
            &[b"sub/x"]
        )
        .expect("parsed"),
        vec![b"sub/x".to_vec()]
    );
    // No record at all is the answer to no question.
    assert_eq!(
        parse_text_unset(b"", &[], &[]).expect("parsed"),
        Vec::<Vec<u8>>::new()
    );
}

/// A reply that does not match the question fails with a malformed-listing cause.
#[test]
fn a_reply_that_does_not_match_the_question_fails() {
    let names: Vec<Vec<u8>> = asked(&["a", "b"]);
    let paths: [&[u8]; 2] = [b"a", b"b"];
    for reply in [
        // Too few records, and too many.
        &b"a\0text\0unset\0"[..],
        b"a\0text\0unset\0b\0text\0unset\0c\0text\0unset\0",
        // A final terminator is missing.
        b"a\0text\0unset\0b\0text\0unset",
        // The records answer other paths, or another attribute, or in another order.
        b"a\0text\0unset\0c\0text\0unset\0",
        b"a\0text\0unset\0b\0eol\0unset\0",
        b"b\0text\0unset\0a\0text\0unset\0",
        b"",
    ] {
        assert_eq!(
            failure_of(parse_text_unset(reply, names.as_slice(), &paths)),
            CandidateFailure::ListingMalformed,
            "{reply:?}"
        );
    }
}

/// No path means no question and no Git process,
///  even when no Git can be started.
#[test]
fn no_path_starts_no_git() {
    let attributes: CandidateAttributes = CandidateAttributes::new(
        Path::new("/nonexistent/git"),
        &[],
        &[],
        AttributeSource::Index,
    );
    assert_eq!(
        attributes.text_unset(&[]).expect("nothing asked"),
        HashSet::<Vec<u8>>::new()
    );
}

/// A Git that cannot be started fails the lookup with the layer's cause for it.
#[test]
fn a_git_that_cannot_be_started_fails_the_lookup() {
    let attributes: CandidateAttributes = CandidateAttributes::new(
        Path::new("/nonexistent/git"),
        &[],
        &[],
        AttributeSource::Index,
    );
    let error: CandidateError = attributes.text_unset(&[b"a"]).expect_err("no Git");
    assert_eq!(error.failure, CandidateFailure::GitNotStarted);
    // The message names the operation and never the candidate's pathname.
    assert!(error.message.contains("git rev-parse could not be started"), "{}", error.message);
    assert!(!error.message.contains("\"a\""), "{}", error.message);
}

/// A path that no commit or tree listed cannot be answered and fails the lookup.
#[test]
fn a_path_without_a_listing_tree_fails_the_lookup() {
    let root: PathBuf = fixture("attribute-unlisted");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let tree: ObjectId = parse_object_id(&[b'a'; 40]).expect("an object name");
    let mut trees: HashMap<Vec<u8>, ObjectId> = HashMap::new();
    trees.insert(b"known".to_vec(), tree);
    let attributes: CandidateAttributes = CandidateAttributes::new(
        Path::new(REAL_GIT),
        &[OsString::from("-C"), repo.as_os_str().to_os_string()],
        &[],
        AttributeSource::PerPath(trees),
    );
    let error: CandidateError = attributes.text_unset(&[b"unknown"]).expect_err("no tree");
    assert_eq!(error.failure, CandidateFailure::StaleCandidate);
    remove(root.as_path());
}

/// A lookup run from a directory outside the worktree,
///  as a command run with `GIT_DIR` and `GIT_WORK_TREE` is,
///  still resolves top-level pathnames.
#[test]
fn a_lookup_from_outside_the_worktree_resolves_top_level_paths() {
    let root: PathBuf = fixture("attribute-outside");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join(".gitattributes"), b"seed/** -text\n").expect("attributes");
    git(repo.as_path(), &["add", ".gitattributes"]);
    let overlay: Vec<(OsString, OsString)> = vec![
        (OsString::from("GIT_DIR"), repo.join(".git").into_os_string()),
        (OsString::from("GIT_WORK_TREE"), repo.as_os_str().to_os_string()),
    ];
    let attributes: CandidateAttributes = CandidateAttributes::new(
        Path::new(REAL_GIT),
        &[],
        overlay.as_slice(),
        AttributeSource::Index,
    );
    // The process's own directory is the package, outside the fixture's worktree.
    let unset: HashSet<Vec<u8>> = attributes
        .text_unset(&[b"seed/case", b"plain.txt"])
        .expect("resolved");
    assert_eq!(unset, HashSet::from([b"seed/case".to_vec()]));
    remove(root.as_path());
}
