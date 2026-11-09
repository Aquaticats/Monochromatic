//! Controls for the landing-location query:
//!  the two answers it reads,
//!  and the answers real Git
//! gives inside a repository,
//!  outside one,
//!  and with an alternate index.

use super::*;
use crate::test_support::{REAL_GIT, fixture, git_context, isolated_overlay, remove, repository};

/// Each answer is one pathname with its own terminating line feed.
#[test]
fn the_two_answers_are_read_in_order() {
    let location: LandingLocation = match landing_location_of(b"/r/.git\n", b"/r/.git/index\n") {
        Ok(found) => found,
        Err(reason) => panic!("two complete answers parse: {reason}"),
    };
    assert_eq!(location.git_dir, PathBuf::from("/r/.git"));
    assert_eq!(location.index_path, PathBuf::from("/r/.git/index"));
    assert_eq!(location.real_index_path, PathBuf::from("/r/.git/index"));
}

/// What:
///  A line feed inside a pathname is part of the pathname,
///  not the end of an answer.
/// Why:
///  One report split at line feeds read the rest of such a name as the index path,
///  so the lock and the transaction registry pointed at names that do not exist.
#[test]
fn a_line_feed_inside_an_answer_is_part_of_the_path() {
    let location: LandingLocation = landing_location_of(b"/r/a\nb/.git\n", b"/r/a\nb/.git/index\n").expect("two complete answers parse");
    assert_eq!(location.git_dir, PathBuf::from("/r/a\nb/.git"), "the Git directory keeps its line feed");
    assert_eq!(location.index_path, PathBuf::from("/r/a\nb/.git/index"), "the index keeps its line feed");
    assert_eq!(location.real_index_path, PathBuf::from("/r/a\nb/.git/index"), "the real index sits beside it");
}

/// An empty answer would point the lock somewhere else,
///  and a second line feed in an answer is a name ending in one.
#[test]
fn an_empty_answer_is_refused_and_a_name_ending_in_a_line_feed_is_one() {
    assert!(landing_location_of(b"", b"/r/.git/index\n").is_err());
    assert!(landing_location_of(b"/r/.git\n", b"").is_err());
    // The one line feed removed is the terminator, so the second one belongs to the directory's own name.
    let location: LandingLocation = landing_location_of(b"/r/.git\n\n", b"/r/.git/index\n").expect("a name ending in a line feed parses");
    assert_eq!(location.git_dir, PathBuf::from("/r/.git\n"), "the name keeps the line feed it ends with");
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
///  A repository whose own directory name holds a line feed is still locked where it is.
/// Why:
///  The layout answer used to be one report split at line feeds,
///  so a line feed in the Git directory's own name ended the first path there and made the rest of it the index:
///  the lock,
///  the transaction registry and the lease directory all pointed at names that do not exist.
///  The in-process reader answers first for an ordinary command,
///  so this control turns it off and measures the answer Git gives.
/// Issue 6007 holds only for a name without a line feed until this answer is read per path.
#[test]
fn a_line_feed_in_the_repository_name_keeps_its_own_paths() {
    let root: PathBuf = fixture("landing-location-line-feed");
    let repo: PathBuf = root.join("a\nb");
    std::fs::create_dir_all(repo.as_path()).expect("repository root");
    crate::test_support::git(repo.as_path(), &["init", "--quiet"]);
    let mut overlay: Vec<(OsString, OsString)> = isolated_overlay();
    overlay.extend(crate::repo_read_hook::switch_off());
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
    assert_eq!(location.git_dir, repo.join(".git"), "the Git directory is the one Git named");
    assert_eq!(location.index_path, repo.join(".git/index"), "the index is where Git writes it");
    assert_eq!(location.real_index_path, repo.join(".git/index"), "the real index sits beside the Git directory's HEAD");
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
    let mut git_dir_answer: Vec<u8> = git_dir.to_vec();
    git_dir_answer.push(b'\n');
    let mut index_answer: Vec<u8> = index.to_vec();
    index_answer.push(b'\n');
    let location: LandingLocation = landing_location_of(git_dir_answer.as_slice(), index_answer.as_slice()).expect("two complete answers parse");
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
