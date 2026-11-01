//! Controls for capture order:
//!  the diff-tree listing parser and every branch of the per-path decision.

use super::*;

/// A landed change over the given paths.
fn landed_change(commit: &str, paths: &[&str]) -> LandedChange {
    return LandedChange {
        commit: String::from(commit),
        paths: paths.iter().map(|path| { return String::from(*path); }).collect(),
    };
}

/// A capture from `worktree` with sequence `sequence` that read the given paths.
fn capture(worktree: &str, sequence: i64, paths: &[&str]) -> CaptureView {
    let owned: Vec<String> = paths.iter().map(|path| { return String::from(*path); }).collect();
    return capture_view(worktree, sequence, &owned);
}

/// Records of the landed commits, keyed by commit.
fn landed_records(entries: &[(&str, CaptureView)]) -> BTreeMap<String, CaptureView> {
    return entries
        .iter()
        .map(|(commit, view)| { return (String::from(*commit), view.clone()); })
        .collect();
}

/// Commits keep their output order; records attach to the commit before them and a commit without records is kept.
#[test]
fn the_listing_attaches_records_to_their_commit() {
    let parsed: Vec<LandedChange> = parse_commit_changes(
        "abc\0:100644 100644 1 2 M\0a.txt\0:100644 100644 1 2 M\0b.txt\0def\0",
    )
    .expect("listing");
    assert_eq!(parsed, vec![landed_change("abc", &["a.txt", "b.txt"]), landed_change("def", &[])]);
    assert!(parse_commit_changes("").expect("empty").is_empty());
}

/// A record before any commit, or a header without its path, is a format error.
#[test]
fn a_malformed_listing_fails() {
    assert!(parse_commit_changes(":100644 x\0a.txt\0").is_err());
    assert!(parse_commit_changes("abc\0:100644 x").is_err());
}

/// A path the prepared capture did not read from the worktree is unordered.
#[test]
fn an_unread_path_is_unordered() {
    let captured: CaptureView = capture("w", 3, &["other.txt"]);
    let history: Vec<LandedChange> = vec![landed_change("c1", &["a.txt"])];
    let landed: BTreeMap<String, CaptureView> = landed_records(&[("c1", capture("w", 1, &["a.txt"]))]);
    assert_eq!(decide_capture_order("a.txt", &captured, &history, &landed), CaptureOrderDecision::Unordered);
}

/// No landed commit changed the path: nothing is ordered.
#[test]
fn a_path_no_landed_commit_changed_is_unordered() {
    let captured: CaptureView = capture("w", 3, &["a.txt"]);
    assert_eq!(decide_capture_order("a.txt", &captured, &[], &BTreeMap::new()), CaptureOrderDecision::Unordered);
}

/// A landed commit without a capture of the path, or from another worktree, leaves the path unordered.
#[test]
fn a_landed_commit_without_a_matching_capture_is_unordered() {
    let captured: CaptureView = capture("w", 3, &["a.txt"]);
    let history: Vec<LandedChange> = vec![landed_change("c1", &["a.txt"])];
    assert_eq!(
        decide_capture_order("a.txt", &captured, &history, &BTreeMap::new()),
        CaptureOrderDecision::Unordered
    );
    let other: BTreeMap<String, CaptureView> = landed_records(&[("c1", capture("elsewhere", 1, &["a.txt"]))]);
    assert_eq!(decide_capture_order("a.txt", &captured, &history, &other), CaptureOrderDecision::Unordered);
    let unread: BTreeMap<String, CaptureView> = landed_records(&[("c1", capture("w", 1, &["b.txt"]))]);
    assert_eq!(decide_capture_order("a.txt", &captured, &history, &unread), CaptureOrderDecision::Unordered);
}

/// A prepared capture later than every landed capture of the path lands its entry;
///  an earlier one keeps the landed entry, judged by the latest landed capture.
#[test]
fn the_later_capture_wins() {
    let history: Vec<LandedChange> = vec![landed_change("c1", &["a.txt"]), landed_change("c2", &["a.txt"])];
    let landed: BTreeMap<String, CaptureView> = landed_records(&[
        ("c1", capture("w", 2, &["a.txt"])),
        ("c2", capture("w", 5, &["a.txt"])),
    ]);
    assert_eq!(
        decide_capture_order("a.txt", &capture("w", 6, &["a.txt"]), &history, &landed),
        CaptureOrderDecision::Prepared
    );
    assert_eq!(
        decide_capture_order("a.txt", &capture("w", 4, &["a.txt"]), &history, &landed),
        CaptureOrderDecision::Landed
    );
}

/// A prepared capture with the same sequence as the latest landed capture is not later, so the landed entry stays.
#[test]
fn an_equal_sequence_is_not_later_than_the_landed_capture() {
    let history: Vec<LandedChange> = vec![landed_change("c1", &["a.txt"])];
    let landed: BTreeMap<String, CaptureView> = landed_records(&[("c1", capture("w", 2, &["a.txt"]))]);
    assert_eq!(
        decide_capture_order("a.txt", &capture("w", 2, &["a.txt"]), &history, &landed),
        CaptureOrderDecision::Landed
    );
}

/// The format error displays exactly its message.
#[test]
fn the_capture_order_error_displays_its_message() {
    let error: CaptureOrderFormatError = CaptureOrderFormatError(String::from("bad listing"));
    assert_eq!(error.to_string(), "bad listing");
}
