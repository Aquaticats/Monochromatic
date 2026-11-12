//! Controls of the file a service's output goes to:
//! its place beside the socket, its cap and its truncation at a start.

use super::*;

fn folder(name: &str) -> std::path::PathBuf {
    let root: std::path::PathBuf = crate::test_support::fixture_root().join(format!("service-log-{name}"));
    std::fs::create_dir_all(&root).expect("fixture folder");
    return root;
}

/// The file is named after the instance beside its socket.
#[test]
fn the_file_is_named_after_the_instance() {
    let root = folder("path");
    assert!(path(root.as_path(), "3f9a").ends_with("3f9a.log"));
}

/// What: A file past its cap is emptied by the next start; one under it keeps its lines.
/// Why:  The last server's lines are what explains a crash, and an unbounded file is a leak.
#[test]
fn a_file_past_its_cap_is_emptied_and_one_under_it_is_kept() {
    let root = folder("cap");
    let log = path(root.as_path(), "3f9a");
    std::fs::write(log.as_path(), "kept\n").expect("a small log");
    truncate_when_past_its_cap(log.as_path()).expect("a small log is not an error");
    assert_eq!(std::fs::read_to_string(log.as_path()).expect("the log reads"), "kept\n");
    let big: String = "x".repeat(usize::try_from(LOG_BYTES_KEPT).expect("a size this host holds") + 1);
    std::fs::write(log.as_path(), big).expect("a log past its cap");
    truncate_when_past_its_cap(log.as_path()).expect("the log is emptied");
    assert_eq!(std::fs::metadata(log.as_path()).expect("the log is there").len(), 0);
}

/// A file that was never written is not a failure to truncate.
#[test]
fn a_file_that_is_not_there_is_not_a_failure() {
    let root = folder("absent");
    truncate_when_past_its_cap(path(root.as_path(), "3f9a").as_path()).expect("nothing to truncate");
}
