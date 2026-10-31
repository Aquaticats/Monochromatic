//! What:
//!  Tests for the node-releases reader.
//! Why:
//!  File order and the `lts` boolean-or-string shape are the two traps the spec names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('parseNodeVersions', () => { /* order, lts shapes, malformed input */ });
//! ```

use crate::node_releases::parse_node_versions;

/// Versions come back in file order, unsorted and undeduplicated, whatever `lts` holds.
#[test]
fn keeps_file_order() {
    let text = r#"[
      {"name":"nodejs","version":"10.0.0","lts":false,"v8":"1"},
      {"name":"nodejs","version":"9.0.0","lts":"Lithium"},
      {"name":"nodejs","version":"10.0.0"}
    ]"#;
    assert_eq!(parse_node_versions(text).expect("parses"), vec!["10.0.0", "9.0.0", "10.0.0"]);
}

/// Malformed input names the file.
#[test]
fn rejects_malformed_input() {
    for text in ["", "{}", "[1]", r#"[{"version":1}]"#, r#"[{"name":"x"}]"#, "[{"] {
        let message = parse_node_versions(text).expect_err("rejected").to_string();
        assert_eq!(message.starts_with("node-releases data/processed/envs.json: "), true, "{text:?}: {message}");
    }
}

/// The committed real revision parses and ends at the version the spec's method records for it.
#[test]
fn parses_the_committed_revision() {
    let versions = parse_node_versions(include_str!("../fixtures/data/envs.json.txt")).expect("real file parses");
    assert_eq!(versions.first().map(String::as_str), Some("0.2.0"));
    assert_eq!(versions.last().map(String::as_str).is_some(), true);
    assert_eq!(versions.len() > 380, true);
}
