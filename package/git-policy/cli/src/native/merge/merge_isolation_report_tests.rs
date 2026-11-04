//! Controls for the isolated merge's findings.

use super::*;

/// The one JSON object a finding prints.
fn parsed(line: &str) -> serde_json::Value {
    assert!(line.ends_with('\n') && line.matches('\n').count() == 1, "{line:?}");
    return serde_json::from_str(line).expect("a JSON line");
}

/// An overlap names the paths and every way forward, and blocks as a core finding.
#[test]
fn the_overlap_finding_names_the_paths_and_the_ways_forward() {
    let paths: Vec<Vec<u8>> = vec![b"src/a.rs".to_vec(), b"b \"q\".txt".to_vec()];
    let value = parsed(overlap_finding("feature", paths.as_slice()).as_str());
    assert_eq!(value["type"], "core-finding");
    assert_eq!(value["coreId"], CORE_ID);
    assert_eq!(value["code"], format!("{CORE_ID}/{OVERLAP_CODE}"));
    let message: &str = value["message"].as_str().expect("message");
    assert!(message.contains("src/a.rs") && message.contains("b \"q\".txt"), "{message}");
    for way in ["wait until", "clean linked worktree", "git stash push"] {
        assert!(message.contains(way), "{way} missing from {message}");
    }
    assert!(message.contains("nothing was changed"));
}

/// A long list of paths is cut after five names and counted.
#[test]
fn many_paths_are_counted() {
    let paths: Vec<Vec<u8>> = (0..8).map(|number| { return format!("p{number}").into_bytes(); }).collect();
    let value = parsed(overlap_finding("feature", paths.as_slice()).as_str());
    let message: &str = value["message"].as_str().expect("message");
    assert!(message.contains("p4") && !message.contains("p5") && message.contains("and 3 more"), "{message}");
}

/// A conflict names its paths and the remedies.
#[test]
fn the_conflict_finding_names_the_paths_and_the_remedies() {
    let paths: Vec<String> = vec![String::from("x.txt"), String::from("y/z.txt")];
    let value = parsed(conflict_finding("feature", paths.as_slice()).as_str());
    assert_eq!(value["code"], format!("{CORE_ID}/{CONFLICT_CODE}"));
    let message: &str = value["message"].as_str().expect("message");
    assert!(message.contains("x.txt, y/z.txt"), "{message}");
    for way in ["clean linked worktree", "clean checkout", "rebase"] {
        assert!(message.contains(way), "{way} missing from {message}");
    }
}

/// A refused last step says nothing changed and what to do.
#[test]
fn the_refused_apply_finding_quotes_gits_reason() {
    let value = parsed(not_applied_finding("feature", "untracked file x").as_str());
    assert_eq!(value["code"], format!("{CORE_ID}/{APPLY_REFUSED_CODE}"));
    let message: &str = value["message"].as_str().expect("message");
    assert!(message.contains("untracked file x") && message.contains("as they were"), "{message}");
}
