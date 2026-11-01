//! Controls for the landing-location query:
//!  the layout parser's refusals,
//!  and the answers real Git
//! gives inside a repository,
//!  outside one,
//!  and with an alternate index.

use super::*;
use crate::test_support::{REAL_GIT, fixture, git_context, isolated_overlay, remove, repository};

/// Layout text with the three fixed lines.
#[test]
fn a_complete_layout_is_parsed_in_order() {
    let location: LandingLocation = match parse_layout(b"/r/.git\n/r/.git/index\n/r/.git/cli-git-transactions\n") {
        Ok(found) => found,
        Err(reason) => panic!("a complete layout parses: {reason}"),
    };
    assert_eq!(location.git_dir, PathBuf::from("/r/.git"));
    assert_eq!(location.index_path, PathBuf::from("/r/.git/index"));
    assert_eq!(location.real_index_path, PathBuf::from("/r/.git/index"));
    assert_eq!(location.registry_root, PathBuf::from("/r/.git/cli-git-transactions"));
}

/// A missing or empty line would point the lock somewhere else,
///  so the parser refuses it.
#[test]
fn an_incomplete_layout_is_refused() {
    assert!(parse_layout(b"").is_err());
    assert!(parse_layout(b"/r/.git\n/r/.git/index\n").is_err());
    assert!(parse_layout(b"/r/.git\n\n/r/.git/cli-git-transactions\n").is_err());
}

/// Inside a repository the query answers with the git directory,
///  the index and the registry.
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
    assert_eq!(location.registry_root, repo.join(".git/cli-git-transactions"));
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
