//! Real ripgrep subprocesses verify bounded results,
//!  independent errors,
//!  native paths,
//!  and latest-query ownership.

/// Disposable fixture directories,
///  in memory when the system has one.
mod memory_fixture;

/// Search replies preserve typed source locations and stream-specific errors.
use ide_app::{
    search::{MAX_CONTENT_RESULTS, MAX_PATH_RESULTS, MAX_SEARCH_RECORD, SearchHit, SearchKind},
    search_worker::{SearchReply, SearchWorker},
    workspace::Workspace,
};
/// Fixtures are disposable and waits are bounded independently of child output timing.
use std::{
    ffi::{OsStr, OsString},
    fs,
    os::unix::ffi::OsStringExt,
    sync::Arc,
    time::{Duration, Instant},
};

/// Whether some hit is the unabridged content match on this line with this preview.
fn has_content_hit(hits: &[SearchHit], line: usize, preview: &str) -> bool {
    let wanted = SearchKind::Content {
        line,
        preview: preview.to_string(),
        truncated: false,
    };
    for hit in hits {
        if hit.kind == wanted {
            return true;
        }
    }
    return false;
}

/// Whether every hit is a content match on the first line of its file.
fn all_on_first_line(hits: &[SearchHit]) -> bool {
    for hit in hits {
        if !matches!(hit.kind, SearchKind::Content { line: 1, .. }) {
            return false;
        }
    }
    return true;
}

/// Whether some hit is a file of exactly this name.
fn has_file_name(hits: &[SearchHit], name: &OsStr) -> bool {
    for hit in hits {
        if hit.path.file_name() == Some(name) {
            return true;
        }
    }
    return false;
}

/// Whether some hit's path ends with these components.
fn has_path_ending(hits: &[SearchHit], tail: &str) -> bool {
    for hit in hits {
        if hit.path.ends_with(tail) {
            return true;
        }
    }
    return false;
}

/// Wait for the currently requested reply without polling a subprocess directly from the GUI boundary.
fn reply(worker: &mut SearchWorker) -> Arc<SearchReply> {
    let start = Instant::now();
    loop {
        if let Some(result) = worker.try_take().expect("search worker remains available") {
            return result;
        }
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "search did not complete"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// Hidden/ignored files stay outside default ripgrep search,
///  while path and content streams retain their own semantics.
#[test]
fn real_search_preserves_path_and_content_semantics() {
    let fixture = memory_fixture::directory("ide-search-worker-");
    fs::write(
        fixture.path().join("Needle-path.txt"),
        "needle first\nneedle second\n",
    )
    .expect("matching fixture");
    fs::write(
        fixture.path().join("other.txt"),
        "unmatched\nneedle elsewhere\n",
    )
    .expect("content fixture");
    fs::write(fixture.path().join(".hidden-needle.txt"), "needle hidden").expect("hidden fixture");
    fs::write(fixture.path().join("ignored-needle.txt"), "needle ignored")
        .expect("ignored fixture");
    fs::write(fixture.path().join(".ignore"), "ignored-needle.txt\n").expect("ignore fixture");
    let workspace = Workspace::new(fixture.path()).expect("workspace");
    let mut worker = SearchWorker::new(workspace).expect("search worker");
    assert!(worker.try_take().expect("idle poll").is_none());
    let generation = worker
        .request("needle".to_string())
        .expect("search request");
    let result = reply(&mut worker);
    assert_eq!(result.generation, generation);
    assert_eq!(result.query, "needle");
    let paths = result.results.paths.as_ref().expect("filename search");
    let contents = result.results.contents.as_ref().expect("content search");
    assert_eq!(paths.len(), 1);
    assert!(paths[0].path.ends_with("Needle-path.txt"));
    assert_eq!(paths[0].kind, SearchKind::Path);
    assert_eq!(contents.len(), 2);
    assert!(has_content_hit(contents, 1, "needle first"));
    assert!(has_content_hit(contents, 2, "needle elsewhere"));
}

/// Each result cap is independent,
///  and only the first matching line in each file enters content results.
#[test]
fn both_result_streams_stop_at_their_approved_caps() {
    let fixture = memory_fixture::directory("ide-search-worker-");
    for index in 0..45 {
        fs::write(
            fixture.path().join(format!("needle-{index}.txt")),
            "needle first\nneedle second\n",
        )
        .expect("bounded source fixture");
    }
    let mut worker =
        SearchWorker::new(Workspace::new(fixture.path()).expect("workspace")).expect("worker");
    worker.request("needle".to_string()).expect("capped search");
    let result = reply(&mut worker);
    assert_eq!(
        result.results.paths.as_ref().expect("path cap").len(),
        MAX_PATH_RESULTS
    );
    let contents = result.results.contents.as_ref().expect("content cap");
    assert_eq!(contents.len(), MAX_CONTENT_RESULTS);
    assert!(all_on_first_line(contents));
}

/// Invalid content regexes retain useful literal filename results instead of silently returning no matches.
#[test]
fn invalid_regex_and_argument_like_queries_remain_data() {
    let fixture = memory_fixture::directory("ide-search-worker-");
    fs::write(fixture.path().join("literal[.txt"), "--hidden\n").expect("punctuation fixture");
    let mut worker =
        SearchWorker::new(Workspace::new(fixture.path()).expect("workspace")).expect("worker");
    worker
        .request("[".to_string())
        .expect("invalid regex request");
    let invalid = reply(&mut worker);
    assert_eq!(
        invalid
            .results
            .paths
            .as_ref()
            .expect("literal filename results")
            .len(),
        1
    );
    assert!(
        invalid
            .results
            .contents
            .as_ref()
            .expect_err("regex diagnostic")
            .to_string()
            .contains("regex parse error")
    );
    worker
        .request("--hidden".to_string())
        .expect("argument-like query");
    let literal = reply(&mut worker);
    assert_eq!(
        literal
            .results
            .contents
            .as_ref()
            .expect("literal dash pattern")
            .len(),
        1
    );
    worker
        .request("absent-pattern".to_string())
        .expect("no match query");
    let empty = reply(&mut worker);
    assert!(
        empty
            .results
            .paths
            .as_ref()
            .expect("no filenames")
            .is_empty()
    );
    assert!(
        empty
            .results
            .contents
            .as_ref()
            .expect("no content matches")
            .is_empty()
    );
}

/// Actual NUL filename output and base64 JSON preserve invalid UTF-8 and newline-containing names.
#[test]
fn subprocess_results_preserve_native_filename_bytes() {
    let fixture = memory_fixture::directory("ide-search-worker-");
    let native = OsString::from_vec(b"needle-\xff.txt".to_vec());
    fs::write(fixture.path().join(&native), "needle native").expect("native byte fixture");
    fs::write(
        fixture.path().join("needle-line\nbreak.txt"),
        "needle newline",
    )
    .expect("newline fixture");
    let mut worker =
        SearchWorker::new(Workspace::new(fixture.path()).expect("workspace")).expect("worker");
    worker
        .request("needle".to_string())
        .expect("native filename search");
    let result = reply(&mut worker);
    for stream in [&result.results.paths, &result.results.contents] {
        let hits = stream.as_ref().expect("native path stream");
        assert_eq!(hits.len(), 2);
        assert!(has_file_name(hits, native.as_os_str()));
        assert!(has_path_ending(hits, "needle-line\nbreak.txt"));
    }
}

/// Replacing and clearing requests prevents obsolete replies from entering the result model.
#[test]
fn latest_generation_wins_and_clear_discards_unread_results() {
    let fixture = memory_fixture::directory("ide-search-worker-");
    fs::write(fixture.path().join("first.txt"), "first value").expect("first fixture");
    fs::write(fixture.path().join("last.txt"), "last value").expect("last fixture");
    let mut worker =
        SearchWorker::new(Workspace::new(fixture.path()).expect("workspace")).expect("worker");
    for _request in 0..20 {
        worker
            .request("first".to_string())
            .expect("superseded request");
    }
    let generation = worker.request("last".to_string()).expect("latest request");
    let current = reply(&mut worker);
    assert_eq!(current.generation, generation);
    assert_eq!(current.query, "last");
    assert!(
        current.results.paths.as_ref().expect("latest paths")[0]
            .path
            .ends_with("last.txt")
    );
    worker
        .request("first".to_string())
        .expect("request before clear");
    worker.clear().expect("clear pending search");
    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(100) {
        assert!(worker.try_take().expect("cleared worker").is_none());
        std::thread::sleep(Duration::from_millis(2));
    }
    worker
        .request("last".to_string())
        .expect("request before shutdown");
    drop(worker);
}

/// A huge matching JSON line fails only the content stream,
///  without allocating an unbounded result or losing path hits.
#[test]
fn oversized_content_record_retains_filename_results() {
    let fixture = memory_fixture::directory("ide-search-worker-");
    let mut line = vec![b'x'; MAX_SEARCH_RECORD + 100];
    line[..6].copy_from_slice(b"needle");
    fs::write(fixture.path().join("needle-long.txt"), line).expect("bounded oversized fixture");
    let mut worker =
        SearchWorker::new(Workspace::new(fixture.path()).expect("workspace")).expect("worker");
    worker
        .request("needle".to_string())
        .expect("long-line search");
    let result = reply(&mut worker);
    assert_eq!(
        result
            .results
            .paths
            .as_ref()
            .expect("retained path result")
            .len(),
        1
    );
    assert!(
        format!(
            "{:#}",
            result.results.contents.as_ref().expect_err("record limit")
        )
        .contains("exceeds the")
    );
}
