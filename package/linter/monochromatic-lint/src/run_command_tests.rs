//! What:
//!  Whole-invocation controls for discovery,
//!  configuration,
//!  output routing and exit status.
//! Why:
//!  Each piece has its own tests;
//!  these prove the pieces are connected the way the command
//! line promises,
//!  by running complete invocations over disposable directory trees.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe(runCommand.name, () => { /* clean, findings, warnings, setup errors, ignores, modes */ });
//! ```

/// Import the shared invocation fixtures.
use crate::run_output::RunOutput;
use crate::run_test_support::{ALL_RULES, CONFIG, codes, read, records, run, write};
use crate::test_fs::Fixture;
use std::path::Path;

/// The `filename` of every record,
///  in output order.
fn filenames(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::<String>::new();
    for record in records(text) {
        found.push(String::from(
            record["filename"].as_str().expect("filename is text"),
        ));
    }
    return found;
}

/// A clean run prints nothing on either stream and exits 0,
///  with or without `--fix`.
#[test]
fn a_clean_run_prints_nothing() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, ALL_RULES);
    write(&fixture.path, "doc/a.md", "# Title\n\nOne sentence.\n");
    write(
        &fixture.path,
        "src/a.rs",
        "//! File.\n\n/// Item.\nfn item() {}\n",
    );
    write(&fixture.path, "notes.txt", "# Not. Linted.\n");
    for arguments in [vec![], vec!["--fix"], vec!["."], vec!["doc", "src/a.rs"]] {
        let output: RunOutput = run(&fixture.path, arguments.as_slice(), "");
        assert_eq!(output.stdout, "", "{arguments:?}");
        assert_eq!(output.stderr, "", "{arguments:?}");
        assert_eq!(output.exit_code, 0, "{arguments:?}");
    }
}

/// Findings are JSONL on standard output in the established record shape,
///  ordered by file then position.
#[test]
fn findings_are_jsonl_in_the_established_shape() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, ALL_RULES);
    write(&fixture.path, "b.md", "# Title.\n\n# Second\n");
    write(&fixture.path, "a/z.rs", "#[test]\nfn main() {}\n");
    let output: RunOutput = run(&fixture.path, &[], "");
    assert_eq!(output.exit_code, 1);
    assert_eq!(output.stderr, "");
    assert!(output.stdout.ends_with('\n'));
    assert_eq!(
        filenames(output.stdout.as_str()),
        ["a/z.rs", "a/z.rs", "a/z.rs", "b.md", "b.md"]
    );
    assert_eq!(
        codes(output.stdout.as_str()),
        [
            "rust/require-rustdoc",
            "rust/require-rustdoc",
            "rust/tests-in-sidecars",
            "markdown/no-trailing-punctuation",
            "markdown/single-h1"
        ]
    );
    // The fourth line, byte for byte: field order and names are the consumers' contract.
    let line: &str = output.stdout.lines().nth(3).expect("fourth record");
    assert_eq!(
        line,
        r#"{"message":"Heading ends with punctuation; remove the trailing punctuation.","code":"markdown/no-trailing-punctuation","severity":"error","causes":[],"filename":"b.md","labels":[{"span":{"offset":0,"length":8,"line":1,"column":1}}],"related":[]}"#
    );
    // Internal fields never reach the wire.
    let record: &serde_json::Value = &records(output.stdout.as_str())[3];
    assert!(record.get("fix").is_none());
    assert!(record.get("processing_failure").is_none());
}

/// Warnings alone do not fail the run;
///  the warning limit,
///  quiet and silent act on accounting and display only.
#[test]
fn warnings_limits_quiet_and_silent_are_independent_of_linting() {
    let fixture: Fixture = Fixture::new();
    write(
        &fixture.path,
        CONFIG,
        r#"[{ "files": ["**/*.md"], "rules": { "markdown/no-trailing-punctuation": { "severity": "warn" }, "markdown/single-h1": { "severity": "error" } } }]"#,
    );
    write(&fixture.path, "warn.md", "# Title.\n");
    let warned: RunOutput = run(&fixture.path, &[], "");
    assert_eq!(warned.exit_code, 0);
    assert_eq!(
        codes(warned.stdout.as_str()),
        ["markdown/no-trailing-punctuation"]
    );
    assert_eq!(records(warned.stdout.as_str())[0]["severity"], "warn");
    assert_eq!(
        run(&fixture.path, &["--max-warnings", "1"], "").exit_code,
        0
    );
    let limited: RunOutput = run(&fixture.path, &["--max-warnings", "0"], "");
    assert_eq!(limited.exit_code, 1);
    assert_eq!(limited.stdout, warned.stdout);
    let quiet: RunOutput = run(&fixture.path, &["--quiet", "--max-warnings", "0"], "");
    assert_eq!(quiet.stdout, "");
    assert_eq!(quiet.exit_code, 1);
    write(&fixture.path, "error.md", "# One\n\n# Two.\n");
    let mixed: RunOutput = run(&fixture.path, &["--quiet"], "");
    assert_eq!(codes(mixed.stdout.as_str()), ["markdown/single-h1"]);
    assert_eq!(mixed.exit_code, 1);
    let silent: RunOutput = run(&fixture.path, &["--silent"], "");
    assert_eq!(silent.stdout, "");
    assert_eq!(silent.stderr, "");
    assert_eq!(silent.exit_code, 1);
}

/// Setup failures print one prefixed line on standard error,
///  nothing on standard output,
///  and exit 2.
#[test]
fn setup_failures_exit_two_with_a_prefixed_explanation() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, "a.md", "# Title.\n");
    let unconfigured: RunOutput = run(&fixture.path, &["--fix"], "");
    assert_eq!(unconfigured.exit_code, 2);
    assert_eq!(unconfigured.stdout, "");
    assert!(
        unconfigured.stderr.starts_with("monochromatic-lint: No monochromatic-lint.config.jsonc was found for any of the 1 input file(s)."),
        "{}",
        unconfigured.stderr
    );
    assert!(unconfigured.stderr.ends_with('\n'));
    // Nothing was rewritten before the setup error.
    assert_eq!(read(&fixture.path, "a.md"), "# Title.\n");
    write(&fixture.path, CONFIG, "[{");
    let malformed: RunOutput = run(&fixture.path, &["--fix"], "");
    assert_eq!(malformed.exit_code, 2);
    assert!(malformed.stderr.contains(CONFIG), "{}", malformed.stderr);
    assert_eq!(read(&fixture.path, "a.md"), "# Title.\n");
    write(&fixture.path, CONFIG, ALL_RULES);
    let unmatched: RunOutput = run(&fixture.path, &["missing/*.md"], "");
    assert_eq!(unmatched.exit_code, 2);
    assert!(
        unmatched
            .stderr
            .contains("matched no supported source files"),
        "{}",
        unmatched.stderr
    );
    let allowed: RunOutput = run(
        &fixture.path,
        &["--no-error-on-unmatched-pattern", "missing/*.md"],
        "",
    );
    assert_eq!(
        (
            allowed.exit_code,
            allowed.stdout.as_str(),
            allowed.stderr.as_str()
        ),
        (0, "", "")
    );
    let absent_config: RunOutput = run(&fixture.path, &["--config", "absent.jsonc"], "");
    assert_eq!(absent_config.exit_code, 2);
    assert!(
        absent_config.stderr.contains("absent.jsonc"),
        "{}",
        absent_config.stderr
    );
    let relative: RunOutput = crate::run_command::run_command(
        &crate::run_test_support::options(&[]),
        Path::new("relative"),
        &mut "".as_bytes(),
    );
    assert_eq!(relative.exit_code, 2);
}

/// A directory with no supported files is clean;
///  partial configuration coverage lints what is configured.
#[test]
fn empty_and_partly_configured_trees_are_not_errors() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, "notes.txt", "nothing to lint\n");
    let empty: RunOutput = run(&fixture.path, &[], "");
    assert_eq!(
        (
            empty.exit_code,
            empty.stdout.as_str(),
            empty.stderr.as_str()
        ),
        (0, "", "")
    );
    write(&fixture.path, "outside.md", "# Title.\n");
    write(&fixture.path, &format!("inside/{CONFIG}"), ALL_RULES);
    write(&fixture.path, "inside/a.md", "# Title.\n");
    let partial: RunOutput = run(&fixture.path, &[], "");
    assert_eq!(partial.exit_code, 1);
    assert_eq!(filenames(partial.stdout.as_str()), ["inside/a.md"]);
}

/// The walker honours ignore files,
///  the built-in exclusions,
///  hidden directories and the ignore flags.
#[test]
fn discovery_honours_ignore_sources_and_flags() {
    let fixture: Fixture = Fixture::new();
    let root: &Path = fixture.path.as_path();
    write(root, CONFIG, ALL_RULES);
    // The walker applies `.gitignore` only inside a Git repository.
    std::fs::create_dir(root.join(".git")).expect("repository marker");
    write(root, ".git/hidden.md", "# Title.\n");
    write(root, ".gitignore", "ignored-by-git/\n");
    write(root, ".ignore", "ignored-by-dot-ignore/\n");
    write(root, "extra.ignore", "ignored-by-path/\n");
    for directory in [
        "kept",
        ".hidden",
        "ignored-by-git",
        "ignored-by-dot-ignore",
        "ignored-by-path",
        "ignored-by-pattern",
        "node_modules/pkg",
    ] {
        write(root, format!("{directory}/a.md").as_str(), "# Title.\n");
    }
    let default: RunOutput = run(root, &[], "");
    assert_eq!(
        filenames(default.stdout.as_str()),
        [
            ".hidden/a.md",
            "ignored-by-path/a.md",
            "ignored-by-pattern/a.md",
            "kept/a.md"
        ]
    );
    let flagged: RunOutput = run(
        root,
        &[
            "--ignore-pattern",
            "ignored-by-pattern/**",
            "--ignore-path",
            "extra.ignore",
        ],
        "",
    );
    assert_eq!(
        filenames(flagged.stdout.as_str()),
        [".hidden/a.md", "kept/a.md"]
    );
    // --no-ignore disables ignore sources; Git metadata is still never walked.
    let unignored: RunOutput = run(root, &["--no-ignore"], "");
    assert_eq!(
        filenames(unignored.stdout.as_str()),
        [
            ".hidden/a.md",
            "ignored-by-dot-ignore/a.md",
            "ignored-by-git/a.md",
            "ignored-by-path/a.md",
            "ignored-by-pattern/a.md",
            "kept/a.md",
            "node_modules/pkg/a.md"
        ]
    );
    // Naming a file explicitly bypasses traversal ignores, but not configuration ignores.
    let explicit: RunOutput = run(root, &["ignored-by-git/a.md"], "");
    assert_eq!(filenames(explicit.stdout.as_str()), ["ignored-by-git/a.md"]);
    write(
        root,
        CONFIG,
        r#"[{ "ignores": ["kept/"] }, { "files": ["**/*.md"], "rules": { "markdown/no-trailing-punctuation": { "severity": "error" } } }]"#,
    );
    let configured: RunOutput = run(root, &["kept/a.md", ".hidden/a.md"], "");
    assert_eq!(filenames(configured.stdout.as_str()), [".hidden/a.md"]);
}

/// The non-linting modes answer through the same entry point and touch no source file.
#[test]
fn rules_init_and_print_config_run_without_linting() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, "a.md", "# Title.\n");
    let rules: RunOutput = run(&fixture.path, &["--rules"], "");
    assert_eq!(rules.exit_code, 0);
    assert_eq!(records(rules.stdout.as_str()).len(), 27);
    assert_eq!(rules.stderr, "");
    let init: RunOutput = run(&fixture.path, &["--init"], "");
    assert_eq!(init.exit_code, 0);
    assert_eq!(
        init.stdout,
        format!("{}\n", fixture.path.join(CONFIG).display())
    );
    let again: RunOutput = run(&fixture.path, &["--init"], "");
    assert_eq!(again.exit_code, 2);
    assert_eq!(again.stdout, "");
    assert!(
        again
            .stderr
            .starts_with("monochromatic-lint: Cannot create "),
        "{}",
        again.stderr
    );
    let printed: RunOutput = run(&fixture.path, &["--print-config", "a.md"], "");
    assert_eq!(printed.exit_code, 0);
    assert!(
        printed.stdout.contains("\"configured\""),
        "{}",
        printed.stdout
    );
    assert!(
        printed.stdout.contains("markdown/single-h1"),
        "{}",
        printed.stdout
    );
    // The starter configuration lints the fixture, and printing it changed nothing on disk.
    assert_eq!(read(&fixture.path, "a.md"), "# Title.\n");
    let linted: RunOutput = run(&fixture.path, &[], "");
    assert_eq!(
        codes(linted.stdout.as_str()),
        ["markdown/no-trailing-punctuation"]
    );
    let broken: RunOutput = run(
        &fixture.path,
        &["--config", "a.md", "--print-config", "a.md"],
        "",
    );
    assert_eq!(broken.exit_code, 2);
}

/// `--debug` adds prefixed lines to standard error and leaves standard output and the status unchanged.
#[test]
fn debug_notes_stay_on_standard_error() {
    let fixture: Fixture = Fixture::new();
    write(
        &fixture.path,
        CONFIG,
        r#"[{ "ignores": ["skip/"] }, { "files": ["**/*.md"], "rules": { "markdown/no-trailing-punctuation": { "severity": "error" } } }]"#,
    );
    write(&fixture.path, "a.md", "# Title.\n");
    write(&fixture.path, "skip/b.md", "# Title.\n");
    let plain: RunOutput = run(&fixture.path, &["--concurrency", "1"], "");
    let debug: RunOutput = run(&fixture.path, &["--concurrency", "1", "--debug"], "");
    assert_eq!(debug.stdout, plain.stdout);
    assert_eq!(debug.exit_code, plain.exit_code);
    assert_eq!(plain.stderr, "");
    for line in debug.stderr.lines() {
        assert!(line.starts_with("monochromatic-lint: debug: "), "{line}");
    }
    assert!(
        debug.stderr.contains("discovered 2 supported file(s)"),
        "{}",
        debug.stderr
    );
    assert!(
        debug.stderr.contains("skip/b.md: ignored by configuration"),
        "{}",
        debug.stderr
    );
    // Exactly the one linted file is named as selected; the ignored one is not.
    assert!(
        debug
            .stderr
            .contains("monochromatic-lint: debug: a.md: selected for linting\n"),
        "{}",
        debug.stderr
    );
    assert_eq!(debug.stderr.matches(": selected for linting").count(), 1);
    assert!(
        debug
            .stderr
            .contains("linting 1 file(s) with a concurrency limit of 1"),
        "{}",
        debug.stderr
    );
    assert!(
        debug
            .stderr
            .contains("rewrote 0 file(s); loaded 0 Cargo workspace(s)"),
        "{}",
        debug.stderr
    );
    let fixed: RunOutput = run(&fixture.path, &["--fix", "--debug"], "");
    assert!(
        fixed.stderr.contains("rewrote 1 file(s)"),
        "{}",
        fixed.stderr
    );
    assert!(
        fixed
            .stderr
            .contains("a.md: fix loop changed the source in 1 pass(es)"),
        "{}",
        fixed.stderr
    );
}

/// A file that cannot be processed fails the run with status 2 while other files are still reported.
#[test]
fn processing_failures_exit_two_and_do_not_hide_other_findings() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, ALL_RULES);
    write(&fixture.path, "a.md", "# Title.\n");
    write(&fixture.path, "broken.mdx", "<A>\ntext\n</B>\n");
    let output: RunOutput = run(&fixture.path, &[], "");
    assert_eq!(output.exit_code, 2);
    assert_eq!(
        codes(output.stdout.as_str()),
        [
            "markdown/no-trailing-punctuation",
            "core/processing-failure"
        ]
    );
    assert_eq!(filenames(output.stdout.as_str()), ["a.md", "broken.mdx"]);
    // Silent output still exits 2: unavailable coverage never looks clean.
    let silent: RunOutput = run(&fixture.path, &["--silent"], "");
    assert_eq!((silent.exit_code, silent.stdout.as_str()), (2, ""));
    // Fixing still fixes the files that can be processed.
    let fixed: RunOutput = run(&fixture.path, &["--fix"], "");
    assert_eq!(fixed.exit_code, 2);
    assert_eq!(read(&fixture.path, "a.md"), "# Title\n");
    assert_eq!(read(&fixture.path, "broken.mdx"), "<A>\ntext\n</B>\n");
}

/// Output is identical at every concurrency limit.
#[test]
fn output_does_not_depend_on_the_concurrency_limit() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, ALL_RULES);
    for index in 0..23 {
        write(
            &fixture.path,
            format!("d{}/f{index}.md", index % 4).as_str(),
            format!("# Title {index}.\n\n# Second {index}\n").as_str(),
        );
    }
    let sequential: RunOutput = run(&fixture.path, &["--concurrency", "1"], "");
    assert_eq!(records(sequential.stdout.as_str()).len(), 46);
    for limit in ["2", "5", "32"] {
        let parallel: RunOutput = run(&fixture.path, &["--concurrency", limit], "");
        assert_eq!(parallel, sequential, "limit {limit}");
    }
    assert_eq!(run(&fixture.path, &[], ""), sequential);
}
