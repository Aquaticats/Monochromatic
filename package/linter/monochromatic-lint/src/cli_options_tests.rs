//! What:
//!  Actual command-grammar controls without filesystem or workspace initialization.
//! Why:
//!  Invalid modes,
//!  missing values and native path bytes must be resolved before linting starts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Parse argv with the real CLI definition and inspect typed results or usage errors.
//! ```

/// Import the production command grammar,
///  the severity an unused-directive finding carries,
///  and the parser's fallible entry point.
use super::CliOptions;
use crate::diagnostic::Severity;
use clap::Parser;
/// Import native argv/path storage.
use std::ffi::OsString;
use std::path::PathBuf;

/// Parse a UTF-8 fixture while retaining the actual operating-system argument interface.
fn parse(arguments: &[&str]) -> Result<CliOptions, clap::Error> {
    let mut values: Vec<OsString> = vec![OsString::from("monochromatic-lint")];
    for argument in arguments {
        values.push(OsString::from(*argument));
    }
    return CliOptions::try_parse_from::<Vec<OsString>, OsString>(values);
}

/// Defaults preserve absence until the runner selects its ordinary path or stdin mode.
#[test]
fn defaults_do_not_invent_a_positional_stdin_conflict() {
    let defaults: CliOptions = parse(&[]).expect("defaults");
    assert!(defaults.paths.is_empty());
    assert!(defaults.config.is_none());
    assert!(defaults.concurrency.is_none());
    assert!(!defaults.stdin);
    let stdin: CliOptions =
        parse(&["--stdin", "--stdin-filename", "doc/notes.mdx", "--fix"]).expect("stdin fixer");
    assert!(stdin.paths.is_empty());
    assert!(stdin.stdin && stdin.fix);
    assert_eq!(stdin.stdin_filename, Some(PathBuf::from("doc/notes.mdx")));
}

/// Repeated ignore flags retain each argv token without a custom delimiter language.
#[test]
fn repeated_paths_and_flags_remain_separate_values() {
    let parsed: CliOptions = parse(&[
        "--config=rules.jsonc",
        "--concurrency",
        "2",
        "--max-warnings=0",
        "--ignore-pattern",
        "first/**",
        "--ignore-pattern",
        "second/**",
        "--ignore-path",
        "one.ignore",
        "--ignore-path",
        "two.ignore",
        "--no-ignore",
        "--quiet",
        "--silent",
        "--debug",
        "--no-error-on-unmatched-pattern",
        "--",
        "--literal.rs",
        "path with spaces.md",
    ])
    .expect("valid typed options");
    assert_eq!(
        parsed.paths,
        [
            PathBuf::from("--literal.rs"),
            PathBuf::from("path with spaces.md")
        ]
    );
    assert_eq!(parsed.ignore_patterns, ["first/**", "second/**"]);
    assert_eq!(
        parsed.ignore_paths,
        [PathBuf::from("one.ignore"), PathBuf::from("two.ignore")]
    );
    assert_eq!(parsed.max_warnings, Some(0));
    assert_eq!(parsed.concurrency.expect("selected workers").get(), 2);
    assert!(parsed.quiet && parsed.silent && parsed.debug && parsed.no_ignore);
}

/// Usage failures do not become partial option values or silently select another mode.
#[test]
fn inconsistent_modes_and_invalid_values_are_rejected() {
    for arguments in [
        vec!["--stdin"],
        vec!["--stdin-filename", "input.rs"],
        vec!["--stdin", "--stdin-filename", "input.rs", "other.rs"],
        vec!["--stdin", "--stdin-filename", "input.rs", "--init"],
        vec!["--rules", "--init"],
        vec!["--fix", "--print-config", "input.rs"],
        vec!["--concurrency", "0"],
        vec!["--concurrency", "many"],
        vec!["--max-warnings=-1"],
        vec!["--config"],
        vec!["--rule", "rust/max-lines"],
        vec!["--unknown"],
    ] {
        assert!(parse(arguments.as_slice()).is_err(), "{arguments:?}");
    }
    assert!(parse(&["--rules"]).expect("rule listing").rules);
    assert!(parse(&["--init"]).expect("configuration initializer").init);
    assert_eq!(
        parse(&["--print-config", "input.rs"])
            .expect("effective file config")
            .print_config,
        Some(PathBuf::from("input.rs"))
    );
}

/// Help and version are parser-controlled terminal outcomes,
///  not lint runs.
#[test]
fn help_and_version_remain_available_without_setup() {
    let help: clap::Error = parse(&["--help"]).expect_err("help request");
    assert_eq!(help.kind(), clap::error::ErrorKind::DisplayHelp);
    assert!(help.to_string().contains("--stdin-filename"));
    // Help is written for the person running the command, not copied from source documentation.
    assert!(
        help.to_string()
            .starts_with("Lint Rust, Markdown and MDX with repository-owned policies.\n\nUsage: monochromatic-lint [OPTIONS] [PATH]...")
    );
    assert!(!help.to_string().contains("In TS you'd write"));
    assert!(!help.to_string().contains("What:"));
    let version: clap::Error = parse(&["--version"]).expect_err("version request");
    assert_eq!(version.kind(), clap::error::ErrorKind::DisplayVersion);
}

/// Unix path bytes survive argv parsing without replacement characters.
#[cfg(unix)]
#[test]
fn native_non_utf8_path_arguments_are_preserved() {
    use std::os::unix::ffi::OsStringExt;
    let raw: OsString = OsString::from_vec(vec![b'n', 255, b'.', b'r', b's']);
    let arguments: Vec<OsString> = vec![OsString::from("monochromatic-lint"), raw.clone()];
    let parsed: CliOptions =
        CliOptions::try_parse_from::<Vec<OsString>, OsString>(arguments).expect("native path");
    assert_eq!(parsed.paths, [PathBuf::from(raw)]);
}

/// The unused-directive options resolve like oxlint's:
///  absent reports nothing,
///  the plain flag warns,
///  and the severity option names its level in both `=` and separate-value forms.
#[test]
fn unused_directive_options_resolve_to_a_severity() {
    assert_eq!(
        parse(&[]).expect("no options").directive_options().unused,
        None
    );
    assert_eq!(
        parse(&["--report-unused-disable-directives"])
            .expect("plain flag")
            .directive_options()
            .unused,
        Some(Severity::Warn)
    );
    assert_eq!(
        parse(&["--report-unused-disable-directives-severity=error"])
            .expect("equals form")
            .directive_options()
            .unused,
        Some(Severity::Error)
    );
    assert_eq!(
        parse(&["--report-unused-disable-directives-severity", "off"])
            .expect("separate form")
            .directive_options()
            .unused,
        None
    );
}

/// The two spellings are alternatives,
///  and only the five known severities are accepted.
#[test]
fn unused_directive_options_reject_a_mixture_and_unknown_severities() {
    let both: clap::Error = parse(&[
        "--report-unused-disable-directives",
        "--report-unused-disable-directives-severity=error",
    ])
    .expect_err("both spellings");
    assert_eq!(both.kind(), clap::error::ErrorKind::ArgumentConflict);
    let unknown: clap::Error =
        parse(&["--report-unused-disable-directives-severity=loud"]).expect_err("unknown severity");
    assert_eq!(unknown.kind(), clap::error::ErrorKind::InvalidValue);
}
