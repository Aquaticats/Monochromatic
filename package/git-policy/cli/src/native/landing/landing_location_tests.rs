//! Controls for the landing-location query:
//!  the layout parser's refusals,
//!  and the answers real Git
//! gives inside a repository,
//!  outside one,
//!  and with an alternate index.

use super::*;
use crate::test_support::{REAL_GIT, fixture, git_context, isolated_overlay, remove, repository};

/// Layout text with the two fixed lines.
#[test]
fn a_complete_layout_is_parsed_in_order() {
    let location: LandingLocation = match parse_layout(b"/r/.git\n/r/.git/index\n") {
        Ok(found) => found,
        Err(reason) => panic!("a complete layout parses: {reason}"),
    };
    assert_eq!(location.git_dir, PathBuf::from("/r/.git"));
    assert_eq!(location.index_path, PathBuf::from("/r/.git/index"));
    assert_eq!(location.real_index_path, PathBuf::from("/r/.git/index"));
}

/// A missing or empty line would point the lock somewhere else,
///  so the parser refuses it.
#[test]
fn an_incomplete_layout_is_refused() {
    assert!(parse_layout(b"").is_err());
    assert!(parse_layout(b"/r/.git\n").is_err());
    assert!(parse_layout(b"/r/.git\n\n").is_err());
}

/// Inside a repository the query answers with the git directory,
///  and the index.
#[test]
fn the_query_answers_inside_a_repository() {
    let root: PathBuf = fixture("landing-location-inside");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let arguments: Vec<OsString> = vec![OsString::from("-C"), repo.as_os_str().to_owned()];
    let answer: Option<LandingLocation> =
        resolve_landing_location(&git_context(), arguments.as_slice()).expect("the query runs");
    let location: LandingLocation = answer.expect("a repository answers");
    assert_eq!(location.git_dir, repo.join(".git"));
    assert_eq!(location.real_index_path, repo.join(".git/index"));
    remove(root.as_path());
}

/// Outside a repository the answer is nothing,
///  not an error.
#[test]
fn the_query_answers_nothing_outside_a_repository() {
    let root: PathBuf = fixture("landing-location-outside");
    let arguments: Vec<OsString> = vec![OsString::from("-C"), root.as_os_str().to_owned()];
    assert_eq!(
        resolve_landing_location(&git_context(), arguments.as_slice()).expect("the query runs"),
        None
    );
    remove(root.as_path());
}

/// With `GIT_INDEX_FILE` set,
///  the index Git writes differs from the real one and the answer says so.
#[test]
fn an_alternate_index_is_reported_apart_from_the_real_one() {
    let root: PathBuf = fixture("landing-location-alternate");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let mut overlay: Vec<(OsString, OsString)> = isolated_overlay();
    overlay.push((OsString::from("GIT_INDEX_FILE"), repo.join("alternate").into_os_string()));
    let context: crate::transaction_git::GitContext = crate::transaction_git::GitContext {
        real_git: PathBuf::from(REAL_GIT),
        overlay,
        global_prefix: Vec::new(),
        common_dir: None,
    };
    let arguments: Vec<OsString> = vec![OsString::from("-C"), repo.as_os_str().to_owned()];
    let location: LandingLocation = resolve_landing_location(&context, arguments.as_slice())
        .expect("the query runs")
        .expect("a repository answers");
    assert_eq!(location.index_path, repo.join("alternate"));
    assert_eq!(location.real_index_path, repo.join(".git/index"));
    remove(root.as_path());
}

/// What:
///  A layout whose paths hold bytes that are not UTF-8 keeps them,
///  so the lock is taken at the place Git named.
/// Why:
///  `path_bytes.rs` states the rule this file must follow:
///  a repository path is bytes,
///  and only a message for a person shows replacement text.
///  A lossy conversion replaced those bytes with U+FFFD,
///  which points the transaction registry,
///  the lease directory and the index-lock proof at names that do not exist.
#[test]
fn a_layout_keeps_the_path_bytes_git_printed() {
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    // 0xEF opens a three-byte sequence and is not a name on its own, so this is not UTF-8 text.
    let git_dir: &[u8] = b"/repos/na\xefve/.git";
    let index: &[u8] = b"/repos/na\xefve/.git/index";
    let mut stdout: Vec<u8> = Vec::new();
    stdout.extend_from_slice(git_dir);
    stdout.push(b'\n');
    stdout.extend_from_slice(index);
    stdout.push(b'\n');
    let location: LandingLocation = parse_layout(stdout.as_slice()).expect("a complete layout parses");
    assert_eq!(location.git_dir.as_os_str().as_bytes(), git_dir, "the git directory keeps its bytes");
    assert_eq!(location.index_path.as_os_str().as_bytes(), index, "the index keeps its bytes");
    let expected_real_index: PathBuf = PathBuf::from(std::ffi::OsString::from_vec(git_dir.to_vec())).join("index");
    assert_eq!(
        location.real_index_path.as_os_str().as_bytes(),
        expected_real_index.as_os_str().as_bytes(),
        "the real index is the git directory's own index, byte for byte"
    );
}

/// What:
///  A real Git prints such a path as bytes and the query answers with them,
///  so a repository whose directory name is not UTF-8 is still locked where it is.
/// Why:
///  The parser control above proves the conversion;
///  this one proves what Git actually prints for such a repository,
///  which is the input the parser receives.
#[test]
fn the_query_answers_for_a_repository_whose_name_is_not_utf8() {
    use std::os::unix::ffi::OsStringExt;
    let root: PathBuf = fixture("landing-location-bytes");
    let named: std::ffi::OsString = std::ffi::OsString::from_vec(b"rep\xefo".to_vec());
    let repo: PathBuf = root.join(named);
    std::fs::create_dir_all(repo.as_path()).expect("repository root");
    crate::test_support::git(repo.as_path(), &["init", "--quiet"]);
    let arguments: Vec<OsString> = vec![OsString::from("-C"), repo.as_os_str().to_owned()];
    let location: LandingLocation = resolve_landing_location(&git_context(), arguments.as_slice())
        .expect("the query runs")
        .expect("a repository answers");
    assert_eq!(location.git_dir, repo.join(".git"));
    assert_eq!(location.real_index_path, repo.join(".git/index"));
    remove(root.as_path());
}
