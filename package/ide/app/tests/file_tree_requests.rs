//! Directory requests are single-use identities independent of document revisions and path reuse.

/// Opaque read requests carry native paths while snapshots retain dirent ordering.
use ide_app::{file_tree::FileTree, workspace::DirectoryEntry};
/// Synthetic roots keep asynchronous-state tests independent of filesystem timing.
use std::path::Path;

/// Build a synthetic child without performing filesystem I/O.
fn child(parent: &str, name: &str, is_directory: bool) -> DirectoryEntry {
    // into owns the filename; join retains the exact native path under the supplied parent.
    return DirectoryEntry {
        name: name.into(),
        path: Path::new(parent).join(name),
        is_directory,
    };
}

/// A pending read is not requested again;
///  cloned tokens identify the same single-use request.
#[test]
fn pending_root_request_is_consumed_once() {
    let root = Path::new("/project");
    let mut tree = FileTree::new(root);
    let request = tree.begin_listing(root).expect("begin root read");
    assert_eq!(request.path(), root);
    assert!(tree.missing_listings().is_empty());
    // Cloning shares the identity allocation, rather than issuing another request.
    let reply = request.clone();
    assert!(
        tree.complete_listing(&reply, Ok(vec![]))
            .expect("current reply")
    );
    assert!(
        !tree
            .complete_listing(&request, Ok(vec![]))
            .expect("duplicate reply")
    );
    assert!(tree.missing_listings().is_empty());
}

/// Superseded successes and failures are ignored before validation or diagnostic propagation.
#[test]
fn superseded_replies_cannot_replace_newer_state() {
    let root = Path::new("/project");
    let mut tree = FileTree::new(root);
    let old = tree.begin_listing(root).expect("first request");
    let current = tree.begin_listing(root).expect("replacement request");
    assert!(
        !tree
            .complete_listing(&old, Ok(vec![child("/project", "old", false)]))
            .expect("stale snapshot")
    );
    assert!(
        !tree
            .complete_listing(&old, Err(anyhow::anyhow!("stale read failure")))
            .expect("stale failure")
    );
    assert!(tree.rows().is_empty());
    assert!(
        tree.complete_listing(&current, Ok(vec![child("/project", "new", false)]))
            .expect("current snapshot")
    );
    assert_eq!(tree.rows()[0].entry.name, "new");
}

/// Equal path text does not let another tree instance's token complete a request.
#[test]
fn request_tokens_are_bound_to_their_producing_tree() {
    let root = Path::new("/project");
    let mut first = FileTree::new(root);
    let mut second = FileTree::new(root);
    let first_request = first.begin_listing(root).expect("first tree request");
    let second_request = second.begin_listing(root).expect("second tree request");
    assert!(
        !second
            .complete_listing(&first_request, Ok(vec![]))
            .expect("foreign token")
    );
    assert!(
        second
            .complete_listing(&second_request, Ok(vec![]))
            .expect("own token")
    );
    assert!(
        first
            .complete_listing(&first_request, Ok(vec![]))
            .expect("other own token")
    );
}

/// Current read failures keep the last good snapshot and release the slot for an explicit retry.
#[test]
fn current_failure_preserves_rows_and_allows_retry() {
    let root = Path::new("/project");
    let mut tree = FileTree::new(root);
    let initial = tree.begin_listing(root).expect("initial request");
    assert!(
        tree.complete_listing(&initial, Ok(vec![child("/project", "keep", false)]))
            .expect("initial snapshot")
    );
    let failed = tree.begin_listing(root).expect("refresh request");
    assert!(
        tree.complete_listing(&failed, Err(anyhow::anyhow!("current read failure")))
            .is_err()
    );
    assert_eq!(tree.rows()[0].entry.name, "keep");
    assert!(
        !tree
            .complete_listing(&failed, Ok(vec![]))
            .expect("consumed failed token")
    );
    let retry = tree.begin_listing(root).expect("retry request");
    assert!(
        tree.complete_listing(&retry, Ok(vec![]))
            .expect("retry snapshot")
    );
    assert!(tree.rows().is_empty());
}

/// Removing then recreating a directory does not make its old in-flight reply current again.
#[test]
fn removed_directory_tokens_stay_stale_after_path_reuse() {
    let root = Path::new("/project");
    let directory = root.join("src");
    let mut tree = FileTree::new(root);
    tree.apply_listing(root, vec![child("/project", "src", true)])
        .expect("root snapshot");
    tree.toggle(&directory).expect("expand src");
    let old = tree
        .begin_listing(&directory)
        .expect("old directory request");
    tree.apply_listing(root, vec![]).expect("directory removal");
    tree.apply_listing(root, vec![child("/project", "src", true)])
        .expect("directory recreation");
    assert!(
        !tree
            .complete_listing(&old, Ok(vec![child("/project/src", "old.rs", false)]))
            .expect("detached old reply")
    );
    tree.toggle(&directory).expect("expand recreated src");
    assert_eq!(tree.missing_listings(), [directory.as_path()]);
    let fresh = tree
        .begin_listing(&directory)
        .expect("fresh directory request");
    assert!(tree.missing_listings().is_empty());
    assert!(
        tree.complete_listing(&fresh, Ok(vec![]))
            .expect("fresh reply")
    );
}

/// A sibling refresh keeps independent pending reads;
///  a synchronous same-directory snapshot supersedes them.
#[test]
fn independent_requests_survive_sibling_updates() {
    let root = Path::new("/project");
    let a = root.join("a");
    let b = root.join("b");
    let mut tree = FileTree::new(root);
    tree.apply_listing(
        root,
        vec![child("/project", "a", true), child("/project", "b", true)],
    )
    .expect("root snapshot");
    let first = tree.begin_listing(&a).expect("a request");
    let second = tree.begin_listing(&b).expect("b request");
    assert!(tree.complete_listing(&first, Ok(vec![])).expect("a reply"));
    assert!(tree.complete_listing(&second, Ok(vec![])).expect("b reply"));
    let replaced = tree.begin_listing(&a).expect("a refresh");
    tree.apply_listing(&a, vec![child("/project/a", "new.rs", false)])
        .expect("synchronous replacement");
    assert!(
        !tree
            .complete_listing(&replaced, Ok(vec![]))
            .expect("superseded refresh")
    );
    assert!(tree.begin_listing(&root.join("missing")).is_err());
    assert!(tree.begin_listing(&a.join("new.rs")).is_err());
}

/// An initial failure restores the missing request;
///  malformed current snapshots preserve existing rows.
#[test]
fn failed_or_invalid_current_replies_release_the_request_slot() {
    let root = Path::new("/project");
    let mut tree = FileTree::new(root);
    let failed = tree.begin_listing(root).expect("initial request");
    assert!(
        tree.complete_listing(&failed, Err(anyhow::anyhow!("missing root")))
            .is_err()
    );
    assert_eq!(tree.missing_listings(), [root]);
    let retry = tree.begin_listing(root).expect("retry");
    assert!(
        tree.complete_listing(&retry, Ok(vec![child("/project", "../escape", false)]))
            .is_err()
    );
    assert_eq!(tree.missing_listings(), [root]);
    assert!(
        !tree
            .complete_listing(&retry, Ok(vec![]))
            .expect("consumed invalid token")
    );
}
