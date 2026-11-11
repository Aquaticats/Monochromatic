//! Controls for the request that selects a private index.

use super::*;
use crate::transaction_git::GitContext;

/// The request carries the private index and object store,
/// and drops the invocation's prefix.
#[test]
fn the_request_selects_the_private_state() {
    let context: GitContext = crate::test_support::git_context();
    let request: GitRequest = indexed_request(&context, Path::new("/work"), &["write-tree"], Some(Path::new("/s/commit.index")), Some(Path::new("/s/objects")));
    assert_eq!(request.cwd, PathBuf::from("/work"));
    assert!(request.without_prefix);
    assert_eq!(request.index, Some(PathBuf::from("/s/commit.index")));
    assert_eq!(request.object_directory, Some(PathBuf::from("/s/objects")));
}

/// Without a private index or object store the request leaves both to Git.
#[test]
fn the_request_may_select_neither() {
    let context: GitContext = crate::test_support::git_context();
    let request: GitRequest = indexed_request(&context, Path::new("/work"), &["status"], None, None);
    assert_eq!(request.index, None);
    assert_eq!(request.object_directory, None);
}
