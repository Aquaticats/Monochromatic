//! What:
//!  The binary tests for the command line itself:
//! help,
//! version,
//! a bad flag,
//! the JSON-line shape of every event,
//! and `explain`.
//! Why:
//! Help,
//! version and parse errors are what a caller sees first,
//! and `explain` is the one command that answers a question without changing a file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe("the command line itself", () => { it("...", () => { ... }); });
//! ```

use crate::support::{
    self,
    CONFIG,
    Harness,
    count_code,
};

#[test]
fn help_version_and_a_bad_flag_are_each_one_json_line_with_exit_codes_zero_zero_two() {
    let harness = Harness::new();
    let (help_code, help) = harness.run(&["--help"]);
    assert_eq!(
        (help_code, help.len(), help[0]["type"].as_str()),
        (0, 1, Some("help"))
    );
    assert!(
        help[0]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("write")
    );
    let (version_code, version) = harness.run(&["--version"]);
    assert_eq!(
        (version_code, version.len(), version[0]["type"].as_str()),
        (0, 1, Some("version"))
    );
    let (bad_code, bad) = harness.run(&["--bogus"]);
    assert_eq!(
        (bad_code, bad.len(), bad[0]["code"].as_str()),
        (2, 1, Some("usage-error"))
    );
    let outside = harness
        .command(&["write"])
        .current_dir(harness.runtime.path())
        .output()
        .expect("runs");
    let missing = support::parse_lines(&String::from_utf8_lossy(&outside.stdout));
    assert_eq!(
        outside.status.code(),
        Some(2),
        "no config anywhere above is a usage error: {missing:?}"
    );
    assert_eq!(missing[0]["code"], "config-missing");
    assert!(
        harness.server_pids().is_empty(),
        "no command above should have started a server"
    );
}

#[test]
fn every_output_line_has_the_event_shape_and_increasing_sequence_numbers() {
    let harness = Harness::new();
    harness.write("file-enforcer.config.jsonc", CONFIG);
    harness.write("AGENTS.md", "body\n");
    let (code, lines) = harness.run(&["check"]);
    assert_eq!(code, 1);
    for (index, line) in lines.iter().enumerate() {
        assert_eq!(line["schemaVersion"], 1);
        assert_eq!(line["sequence"], index);
        assert!(
            line["type"].is_string() && line["code"].is_string() && line["message"].is_string(),
            "{line}"
        );
    }
}

#[test]
fn explain_names_the_rule_that_generates_a_path() {
    let harness = Harness::new();
    harness.write("file-enforcer.config.jsonc", CONFIG);
    harness.write("AGENTS.md", "body\n");
    let (code, lines) = harness.run(&["explain", "CLAUDE.md"]);
    assert_eq!(code, 0);
    assert_eq!(count_code(&lines, "explain-output"), 1, "{lines:?}");
    assert_eq!(lines[0]["rule"], "claude");
}
