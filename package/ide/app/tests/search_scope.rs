//! Selected-directory search narrows the fixed project rather than creating an unrestricted second root.

/// Disposable fixture directories,
///  in memory when the system has one.
mod memory_fixture;

/// The public worker API owns cancellation and background directory validation.
use ide_app::{
    search_worker::{SearchReply, SearchWorker},
    workspace::Workspace,
};
/// Native symlink fixtures stay within a disposable parent directory.
use std::{
    fs,
    os::unix::fs::symlink,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

/// Wait only for the newest scoped reply with a bounded failure deadline.
fn reply(worker: &mut SearchWorker) -> Arc<SearchReply> {
    let start = Instant::now();
    loop {
        if let Some(result) = worker.try_take().expect("search worker") {
            return result;
        }
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "scoped search did not finish"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// Equal queries in different scopes retain the newest scope and never include a sibling's results.
#[test]
fn selected_directory_is_validated_and_bound_to_query_generation() {
    let fixture = memory_fixture::directory("ide-search-scope-");
    for directory in ["a", "b"] {
        fs::create_dir(fixture.path().join(directory)).expect("subdirectory");
        fs::write(
            fixture.path().join(directory).join("needle.txt"),
            format!("needle {directory}"),
        )
        .expect("source fixture");
    }
    let workspace = Workspace::new(fixture.path()).expect("workspace");
    let expected = workspace.root().join("b");
    let mut worker = SearchWorker::new(workspace).expect("worker");
    worker
        .request_scoped("needle".to_string(), PathBuf::from("a"))
        .expect("first scope");
    let generation = worker
        .request_scoped("needle".to_string(), PathBuf::from("b"))
        .expect("newest scope");
    let result = reply(&mut worker);
    assert_eq!(result.generation, generation);
    assert_eq!(result.scope, expected);
    assert_eq!(
        result.results.paths.as_ref().expect("scoped paths").len(),
        1
    );
    assert_eq!(
        result
            .results
            .contents
            .as_ref()
            .expect("scoped contents")
            .len(),
        1
    );
    assert!(
        result.results.paths.as_ref().expect("scoped path")[0]
            .path
            .starts_with(&expected)
    );
    worker.request("needle".to_string()).expect("whole project");
    assert_eq!(
        reply(&mut worker)
            .results
            .paths
            .as_ref()
            .expect("root paths")
            .len(),
        2
    );
}

/// Outside links,
///  parent escapes,
///  files,
///  and missing scopes yield diagnostics without starting an outside search.
#[test]
fn invalid_scope_is_an_error_and_a_later_valid_scope_recovers() {
    let fixture = memory_fixture::directory("ide-search-scope-");
    let root = fixture.path().join("project");
    let outside = fixture.path().join("outside");
    fs::create_dir(&root).expect("project root");
    fs::create_dir(&outside).expect("outside directory");
    fs::write(root.join("needle.txt"), "needle inside").expect("inside fixture");
    fs::write(outside.join("needle.txt"), "needle outside").expect("outside fixture");
    symlink(&outside, root.join("escape")).expect("outside directory alias");
    let mut worker = SearchWorker::new(Workspace::new(&root).expect("workspace")).expect("worker");
    for path in [
        PathBuf::from("escape"),
        PathBuf::from("../outside"),
        outside,
        PathBuf::from("needle.txt"),
        PathBuf::from("missing"),
    ] {
        worker
            .request_scoped("needle".to_string(), path)
            .expect("scope validation stays on worker");
        let result = reply(&mut worker);
        assert!(result.results.paths.is_err());
        assert!(result.results.contents.is_err());
    }
    worker
        .request_scoped("needle".to_string(), root)
        .expect("valid scope after errors");
    assert_eq!(
        reply(&mut worker)
            .results
            .paths
            .as_ref()
            .expect("recovered path search")
            .len(),
        1
    );
}
