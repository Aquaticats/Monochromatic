//! Controls for the isolated merge's findings.

use super::*;

/// The one JSON object a finding prints.
fn parsed(line: &str) -> serde_json::Value {
    assert!(line.ends_with('\n') && line.matches('\n').count() == 1, "{line:?}");
    return serde_json::from_str(line).expect("a JSON line");
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
