//! What:
//!  The binary tests for what a check writes and which drift it reports.
//! Why:
//! `--expected-dir` writes the pair of trees a diff tool reads,
//! and `--outputs` narrows the drift a run reports to the committed or the local side.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe("what a check writes and which drift it reports", () => { it("...", () => { ... }); });
//! ```

use crate::support::{
    CONFIG,
    Harness,
};

#[test]
fn check_with_an_expected_dir_writes_paired_trees_for_diff_tools() {
    let harness = Harness::new();
    harness.write("file-enforcer.config.jsonc", CONFIG);
    harness.write("AGENTS.md", "body\n");
    harness.write("CLAUDE.md", "stale\n");
    let diff_dir = harness.home.path().join("diff");
    let diff_arg = diff_dir.display().to_string();
    let (code, lines) = harness.run(&["check", "--expected-dir", &diff_arg]);
    assert_eq!(code, 1, "{lines:?}");
    assert_eq!(
        std::fs::read_to_string(diff_dir.join("actual/CLAUDE.md")).expect("actual"),
        "stale\n"
    );
    assert_eq!(
        std::fs::read_to_string(diff_dir.join("expected/CLAUDE.md")).expect("expected"),
        "head\nbody\n"
    );
}

/// One committed output and one output the repository's ignore rules cover.
const SPLIT_CONFIG: &str = r#"{"rules": [
    {"kind": "concat", "id": "committed-rule", "output": "committed.txt", "parts": ["c\n"]},
    {"kind": "concat", "id": "local-rule", "output": "out.local.txt", "parts": ["l\n"]}]}"#;

/// The `path` members of the drift lines.
fn drift_paths(lines: &[serde_json::Value]) -> Vec<&str> {
    let mut paths: Vec<&str> = Vec::new();
    for line in lines {
        if line["type"] == "drift" {
            paths.push(line["path"].as_str().expect("a drift names its path"));
        }
    }
    return paths;
}

#[test]
fn outputs_limits_check_and_write_to_committed_or_local_files() {
    let harness = Harness::new();
    harness.write("file-enforcer.config.jsonc", SPLIT_CONFIG);
    harness.write(".gitignore", "*.local.*\n");
    let (all_code, all) = harness.run(&["check"]);
    assert_eq!(
        (all_code, drift_paths(&all)),
        (1, vec!["committed.txt", "out.local.txt"]),
        "{all:?}"
    );
    let (committed_code, committed) = harness.run(&["check", "--outputs", "committed"]);
    assert_eq!(
        (committed_code, drift_paths(&committed)),
        (1, vec!["committed.txt"]),
        "{committed:?}"
    );
    let (local_code, local) = harness.run(&["check", "--outputs=local"]);
    assert_eq!(
        (local_code, drift_paths(&local)),
        (1, vec!["out.local.txt"]),
        "{local:?}"
    );
    assert_eq!(local.last().expect("a result line")["outputs"], "local");
    let (written_code, written) = harness.run(&["write", "--outputs", "committed"]);
    assert_eq!(written_code, 0, "{written:?}");
    assert_eq!(harness.read("committed.txt"), "c\n");
    assert!(
        !harness.repo.path().join("out.local.txt").exists(),
        "the local output is not written by a committed-only write"
    );
    let (clean_code, clean) = harness.run(&["check", "--outputs", "committed"]);
    assert_eq!(
        clean_code, 0,
        "the missing local output is outside the scope: {clean:?}"
    );
    assert_eq!(clean.last().expect("a result line")["code"], "check-clean");
    let (still_code, still) = harness.run(&["check", "--outputs", "all"]);
    assert_eq!(
        (still_code, drift_paths(&still)),
        (1, vec!["out.local.txt"]),
        "{still:?}"
    );
    let (local_write_code, local_write) = harness.run(&["write", "--outputs", "local"]);
    assert_eq!(local_write_code, 0, "{local_write:?}");
    assert_eq!(harness.read("out.local.txt"), "l\n");
    let (final_code, final_lines) = harness.run(&["check", "--outputs", "local"]);
    assert_eq!(final_code, 0, "{final_lines:?}");
}

#[test]
fn outputs_rejects_unknown_words_repeats_and_the_user_commands_as_usage_errors() {
    let harness = Harness::new();
    harness.write("file-enforcer.config.jsonc", SPLIT_CONFIG);
    for arguments in [
        vec!["check", "--outputs", "bogus"],
        vec!["check", "--outputs"],
        vec!["write", "--outputs", "local", "--outputs", "committed"],
        vec!["explain", "--outputs", "local", "committed.txt"],
        vec!["user", "check", "--outputs", "local"],
    ] {
        let (code, lines) = harness.run(&arguments);
        assert_eq!(
            (code, lines.len(), lines[0]["code"].as_str()),
            (2, 1, Some("usage-error")),
            "{arguments:?}: {lines:?}"
        );
    }
    let (help_code, help) = harness.run(&["check", "--help"]);
    assert_eq!(help_code, 0);
    let text = help[0]["message"].as_str().unwrap_or_default();
    assert!(
        text.contains("--outputs") && text.contains("committed") && text.contains("local"),
        "{text}"
    );
    assert!(
        harness.server_pids().is_empty(),
        "a usage error never starts a server"
    );
}
