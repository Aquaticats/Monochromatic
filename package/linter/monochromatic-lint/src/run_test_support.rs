//! What:
//!  Shared fixtures for the orchestration tests.
//! Why:
//!  Every `run_*` test drives the production path over a disposable directory tree;
//!  building
//! that tree,
//!  planning a file and decoding JSONL are the same few steps each time.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // write(root, 'doc/a.md', text); const output = run(root, ['--fix', 'doc'], '');
//! ```

/// Import the production command grammar,
///  runner,
///  planner and output model.
use crate::{
    cli_options::CliOptions,
    run_command::run_command,
    run_output::RunOutput,
    run_plan::{ConfigStore, FilePlan, Planned},
};
/// Import the argument parser trait that provides `try_parse_from`.
use clap::Parser;
/// Import native argument and path storage.
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

/// The configuration file name,
///  repeated here so fixtures do not depend on lookup internals.
pub(crate) const CONFIG: &str = "monochromatic-lint.config.jsonc";

/// A configuration selecting every syntax-only Rust rule and every Markdown rule except the LFS rule.
pub(crate) const ALL_RULES: &str = r#"[
  { "files": ["**/*.rs"], "rules": {
    "rust/max-lines": { "severity": "error", "max": 300 },
    "rust/require-rustdoc": { "severity": "error" },
    "rust/tests-in-sidecars": { "severity": "error" }
  } },
  { "files": ["**/*.md", "**/*.mdx"], "rules": {
    "markdown/heading-increment": { "severity": "error" },
    "markdown/commands-show-output": { "severity": "error" },
    "markdown/no-duplicate-heading": { "severity": "error" },
    "markdown/single-h1": { "severity": "error" },
    "markdown/no-trailing-punctuation": { "severity": "error" },
    "markdown/no-bare-urls": { "severity": "error" },
    "markdown/no-emphasis-as-heading": { "severity": "error" },
    "markdown/fenced-code-language": { "severity": "error" },
    "markdown/link-image-reference-definitions": { "severity": "error" },
    "markdown/link-image-style": { "severity": "error" },
    "markdown/no-pipe-tables": { "severity": "error" },
    "markdown/semantic-line-breaks": { "severity": "error" }
  } }
]"#;

/// Write one file below a root,
///  creating its parent directories.
pub(crate) fn write(root: &Path, relative: &str, contents: &str) -> PathBuf {
    let path: PathBuf = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("fixture directories");
    }
    std::fs::write(&path, contents).expect("fixture file");
    return path;
}

/// Read one fixture file back as text.
pub(crate) fn read(root: &Path, relative: &str) -> String {
    return std::fs::read_to_string(root.join(relative)).expect("fixture file is readable text");
}

/// Parse a command line with the production grammar.
pub(crate) fn options(arguments: &[&str]) -> CliOptions {
    let mut values: Vec<OsString> = vec![OsString::from("monochromatic-lint")];
    for argument in arguments {
        values.push(OsString::from(*argument));
    }
    return CliOptions::try_parse_from::<Vec<OsString>, OsString>(values)
        .expect("fixture command line is valid");
}

/// Run one invocation in a working directory with the given standard input.
pub(crate) fn run(cwd: &Path, arguments: &[&str], stdin: &str) -> RunOutput {
    let parsed: CliOptions = options(arguments);
    let mut input: &[u8] = stdin.as_bytes();
    return run_command(&parsed, cwd, &mut input);
}

/// Decode JSONL into one value per non-empty line;
///  a non-JSON line fails the test.
pub(crate) fn records(text: &str) -> Vec<serde_json::Value> {
    let mut values: Vec<serde_json::Value> = Vec::<serde_json::Value>::new();
    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        values.push(
            serde_json::from_str::<serde_json::Value>(line).expect("one JSON object per line"),
        );
    }
    return values;
}

/// The `code` of every record,
///  in output order.
pub(crate) fn codes(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::<String>::new();
    for record in records(text) {
        found.push(String::from(record["code"].as_str().expect("code is text")));
    }
    return found;
}

/// Plan one file through the production store,
///  with nearest-file lookup from the working directory.
pub(crate) fn plan(cwd: &Path, relative: &str) -> FilePlan {
    let mut store: ConfigStore = ConfigStore::new(cwd, None).expect("store");
    match store.plan(Path::new(relative)).expect("planning succeeds") {
        Planned::Lint { plan: planned } => return *planned,
        Planned::Ignored => panic!("{relative} is ignored by the fixture configuration"),
        Planned::NoConfiguration => panic!("{relative} has no fixture configuration"),
        Planned::Unsupported => panic!("{relative} has an unsupported extension"),
    }
}
