//! Bounded background reads update only current tree requests in disposable workspaces.

/// Disposable fixture directories,
///  in memory when the system has one.
mod memory_fixture;

/// Expected reader failures remain ordinary results in this test helper.
use anyhow::Result;
/// The consumer combines the read-only workspace,
///  UI-thread tree,
///  and background reader.
use ide_app::{directory_worker::DirectoryWorker, file_tree::FileTree, workspace::Workspace};
/// Filesystem fixtures are private;
///  worker waits have an explicit deadline.
use std::{
    fs,
    time::{Duration, Instant},
};

/// Wait for one bounded reply;
///  idle polls never block the caller.
fn finish(worker: &mut DirectoryWorker, tree: &mut FileTree) -> Result<bool> {
    let start = Instant::now();
    while worker.is_busy() {
        // What: ? propagates a current read failure; a stale reply instead returns false successfully.
        // Why: The helper distinguishes an actionable failure from an intentionally discarded snapshot.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const changed = worker.poll(tree);
        // if (!worker.isBusy()) return changed;
        // ```
        let changed = worker.poll(tree)?;
        if !worker.is_busy() {
            return Ok(changed);
        }
        assert!(
            start.elapsed() < Duration::from_secs(3),
            "directory worker did not reply"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
    return Ok(false);
}

/// Busy admission must not supersede the first token or enqueue another directory snapshot.
#[test]
fn reader_bounds_requests_and_preserves_snapshot_order() {
    let fixture = memory_fixture::directory("ide-directory-worker-");
    fs::write(fixture.path().join("猫.rs"), "source").expect("source fixture");
    fs::write(fixture.path().join(".hidden"), "hidden").expect("hidden fixture");
    let workspace = Workspace::new(fixture.path()).expect("workspace");
    let expected = workspace
        .list(workspace.root())
        .expect("reference snapshot");
    // Copy the native root before moving the workspace into its owned worker thread.
    let root = workspace.root().to_path_buf();
    let mut tree = FileTree::new(&root);
    let mut worker = DirectoryWorker::new(workspace).expect("directory worker");
    assert!(!worker.is_busy());
    assert!(!worker.poll(&mut tree).expect("idle poll"));
    assert!(worker.request(&mut tree, &root).expect("first request"));
    assert!(worker.is_busy());
    let second = worker.request(&mut tree, &root);
    let no_duplicate_listing = tree.missing_listings().is_empty();
    // Drain before asserting admission, so a guard-removal regression cannot deadlock Drop behind a full reply slot.
    let first = finish(&mut worker, &mut tree);
    assert!(!second.expect("bounded second request"));
    assert!(no_duplicate_listing);
    assert!(first.expect("first reply still current"));
    // Consume the visible rows and compare their exact native entries, not lossy display strings.
    let mut actual = Vec::new();
    for row in tree.rows() {
        actual.push(row.entry);
    }
    assert_eq!(actual, expected);
    assert!(!worker.is_busy());
}

/// A synchronous replacement makes the in-flight read stale without requiring thread cancellation.
#[test]
fn stale_worker_reply_does_not_replace_current_rows() {
    let fixture = memory_fixture::directory("ide-directory-worker-");
    fs::write(fixture.path().join("old.rs"), "source").expect("source fixture");
    let workspace = Workspace::new(fixture.path()).expect("workspace");
    let root = workspace.root().to_path_buf();
    let mut tree = FileTree::new(&root);
    let mut worker = DirectoryWorker::new(workspace).expect("directory worker");
    worker.request(&mut tree, &root).expect("request read");
    tree.apply_listing(&root, vec![]).expect("newer snapshot");
    assert!(!finish(&mut worker, &mut tree).expect("discard stale reply"));
    assert!(tree.rows().is_empty());
    assert!(!worker.is_busy());
}

/// A current read error retains prior rows and permits a successful read after directory restoration.
#[test]
fn reader_failure_retains_snapshot_and_recovers() {
    let fixture = memory_fixture::directory("ide-directory-worker-");
    let path = fixture.path().join("project");
    fs::create_dir(&path).expect("project directory");
    fs::write(path.join("old.rs"), "old").expect("source fixture");
    let workspace = Workspace::new(&path).expect("workspace");
    let root = workspace.root().to_path_buf();
    let mut tree = FileTree::new(&root);
    let mut worker = DirectoryWorker::new(workspace).expect("directory worker");
    worker.request(&mut tree, &root).expect("initial request");
    assert!(finish(&mut worker, &mut tree).expect("initial reply"));
    fs::remove_file(path.join("old.rs")).expect("external fixture removal");
    fs::remove_dir(&path).expect("external directory removal");
    worker
        .request(&mut tree, &root)
        .expect("missing directory request");
    assert!(finish(&mut worker, &mut tree).is_err());
    assert!(!worker.is_busy());
    assert_eq!(tree.rows()[0].entry.name, "old.rs");
    fs::create_dir(&path).expect("restore directory");
    fs::write(path.join("new.rs"), "new").expect("replacement source");
    worker.request(&mut tree, &root).expect("recovery request");
    assert!(finish(&mut worker, &mut tree).expect("recovery reply"));
    assert_eq!(tree.rows()[0].entry.name, "new.rs");
}

/// The read-only workspace still rejects a request from a tree rooted outside its project boundary.
#[test]
fn workspace_boundary_rejects_foreign_tree_paths() {
    let inside = memory_fixture::directory("ide-directory-worker-");
    let outside = memory_fixture::directory("ide-directory-worker-");
    let workspace = Workspace::new(inside.path()).expect("workspace");
    let foreign = Workspace::new(outside.path()).expect("outside root");
    let mut tree = FileTree::new(foreign.root());
    let mut worker = DirectoryWorker::new(workspace).expect("directory worker");
    worker
        .request(&mut tree, foreign.root())
        .expect("queued foreign root");
    assert!(finish(&mut worker, &mut tree).is_err());
    assert!(tree.rows().is_empty());
    assert!(!worker.is_busy());
}

/// Invalid rows fail before occupying the queue;
///  dropping busy and idle readers joins their threads.
#[test]
fn invalid_admission_and_shutdown_do_not_leave_background_work() {
    let fixture = memory_fixture::directory("ide-directory-worker-");
    let workspace = Workspace::new(fixture.path()).expect("workspace");
    let root = workspace.root().to_path_buf();
    let mut tree = FileTree::new(&root);
    // Clone shares the same canonical boundary while two independently owned workers exercise shutdown states.
    let mut worker = DirectoryWorker::new(workspace.clone()).expect("directory worker");
    assert!(worker.request(&mut tree, &root.join("unknown")).is_err());
    assert!(!worker.is_busy());
    assert!(
        worker
            .request(&mut tree, &root)
            .expect("request before shutdown")
    );
    // Drop closes requests and joins even while the sole response may remain unread.
    drop(worker);
    drop(DirectoryWorker::new(workspace).expect("idle reader"));
}
