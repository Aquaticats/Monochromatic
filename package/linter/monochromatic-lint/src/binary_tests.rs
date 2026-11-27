//! What:
//!  Black-box controls that run the built `monochromatic-lint` executable.
//! Why:
//!  Unit tests passing is not verification of a command-line program.
//!  These tests start the
//! real binary as a child process in disposable directories and assert on its actual exit status,
//! standard output and standard error bytes,
//!  and on what it left on disk.
//! They run inside the bounded,
//!  mount-free,
//!  network-disabled test container with the other tests.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const { status, stdout, stderr } = spawnSync(binary, args, { cwd, input });
//! ```

/// Import child-process control,
///  stream plumbing and native paths.
use std::{
    io::Write,
    path::{
        Path,
        PathBuf,
    },
    process::{
        Child,
        Command,
        Output,
        Stdio,
    },
    sync::atomic::{
        AtomicU64,
        Ordering,
    },
};

/// The declared long options of the command grammar,
///  the help summary it renders,
///  and the program name its output carries.
use monochromatic_lint::cli_options::{
    ABOUT,
    long_options,
};
/// The configuration filename the executable discovers.
use monochromatic_lint::config_lookup::CONFIG_NAME;
/// The largest bound that rule reads one tracked image under,
///  because it reads one byte past the bound.
use monochromatic_lint::markdown_lfs_context::MAX_IMAGE_BYTES_BOUND;
/// The default bound the `markdown/lfs-image-url` rule reads one tracked image under.
use monochromatic_lint::markdown_rule_settings::DEFAULT_MAX_IMAGE_BYTES;
/// The core diagnostic codes the controls expect.
use monochromatic_lint::run_failure::{
    FIX_REFUSED,
    PROCESSING_FAILURE,
};
/// The program name the executable's output carries.
use monochromatic_lint::run_finish::PROGRAM;
/// The shipped rule registry the listing controls compare against.
use monochromatic_lint::run_modes::RULES as REGISTRY;
/// The lockfile name,
/// table header and diagnostic codes the controls expect,
///  re-exported by the library for its executable tests.
use monochromatic_lint::{
    LOCK_NAME,
    NON_SUPPRESSIBLE,
    TOOL_TABLE,
    UNUSED,
};

/// The path of the executable Cargo built for this test run.
const BINARY: &str = env!("CARGO_BIN_EXE_monochromatic-lint");

/// The configuration file name the executable looks up.
const CONFIG: &str = CONFIG_NAME;

/// A configuration selecting the syntax-only Rust rules and the Markdown rules that need no repository.
const RULES: &str = r#"[
  { "files": ["**/*.rs"], "ignores": ["**/*.md/**", "**/*.mdx/**"], "rules": {
    "rust/max-lines": { "severity": "error", "max": 300 },
    "rust/require-rustdoc": { "severity": "error" },
    "rust/tests-in-sidecars": { "severity": "error" }
  } },
  { "files": ["**/*.md", "**/*.mdx"], "ignores": ["**/*.rs/**"], "rules": {
    "markdown/heading-increment": { "severity": "error" },
    "markdown/single-h1": { "severity": "error" },
    "markdown/no-trailing-punctuation": { "severity": "error" },
    "markdown/fenced-code-language": { "severity": "error" },
    "markdown/no-pipe-tables": { "severity": "error" }
  } }
]"#;

/// A process-wide sequence for unique fixture directory names.
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// A directory this test created and removes when it goes out of scope.
struct Fixture {
    /// Root created exclusively for one test.
    path: PathBuf,
}

/// Create and own one disposable directory.
impl Fixture {
    /// Exclusively create a fresh directory under the system temporary directory.
    fn new() -> Fixture {
        let sequence: u64 = SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path: PathBuf = crate::test_scratch::scratch_root().join(format!(
            "monochromatic-lint-binary-{}-{sequence}",
            std::process::id()
        ));
        std::fs::create_dir(&path).expect("exclusively create fixture directory");
        // Resolve symbolic links in the temporary root so reported absolute paths compare exactly.
        return Fixture {
            path: std::fs::canonicalize(&path).expect("resolve fixture directory"),
        };
    }

    /// Write one file below the root,
    ///  creating its parent directories.
    fn write(
        &self,
        relative: &str,
        contents: &str,
    ) -> PathBuf {
        let path: PathBuf = self.path.join(relative);
        std::fs::create_dir_all(path.parent().expect("fixture file has a parent"))
            .expect("fixture directories");
        std::fs::write(&path, contents).expect("fixture file");
        return path;
    }

    /// Read one file below the root as text.
    fn read(
        &self,
        relative: &str,
    ) -> String {
        return std::fs::read_to_string(self.path.join(relative)).expect("fixture text");
    }
}

/// Remove the directory this fixture created,
///  including after a failed assertion.
impl Drop for Fixture {
    /// Delete only the path this instance created.
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.path).expect("remove fixture directory");
    }
}

/// What one invocation produced.
struct Run {
    /// Exit status;
    ///  a signal death has none and fails the test.
    status: i32,
    /// Standard output as text.
    stdout: String,
    /// Standard error as text.
    stderr: String,
}

/// Run the executable in a directory with arguments and standard input,
///  and wait for it.
fn run(
    cwd: &Path,
    arguments: &[&str],
    input: &[u8],
) -> Run {
    let mut child: Child = Command::new(BINARY)
        .args(arguments)
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start the built executable");
    // Dropping the handle after writing closes the pipe, so the child sees end of input.
    let mut stdin = child.stdin.take().expect("piped standard input");
    stdin.write_all(input).expect("write standard input");
    drop(stdin);
    let output: Output = child.wait_with_output().expect("wait for the executable");
    return Run {
        status: output
            .status
            .code()
            .expect("the executable exits; it is not killed by a signal"),
        stdout: String::from_utf8(output.stdout).expect("standard output is UTF-8"),
        stderr: String::from_utf8(output.stderr).expect("standard error is UTF-8"),
    };
}

/// Decode JSONL:
///  every non-empty line must be one JSON object.
fn records(text: &str) -> Vec<serde_json::Value> {
    let mut values: Vec<serde_json::Value> = Vec::<serde_json::Value>::new();
    for line in text.lines() {
        let value: serde_json::Value =
            serde_json::from_str::<serde_json::Value>(line).expect("one JSON object per line");
        assert!(value.is_object(), "{line}");
        values.push(value);
    }
    return values;
}

/// The `--rules` rows agree with the shipped registry,
///  in order and in every capability.
fn assert_listing_matches_registry(listed: &[serde_json::Value]) {
    assert_eq!(listed.len(), REGISTRY.len());
    for (row, rule) in listed.iter().zip(REGISTRY) {
        assert_eq!(row["id"], rule.id);
        assert_eq!(row["fixable"], rule.fixable);
        assert_eq!(row["suppressible"], rule.suppressible);
        let options: Vec<&str> = row["options"]
            .as_array()
            .expect("options")
            .iter()
            .map(|option| return option.as_str().expect("option name"))
            .collect();
        assert_eq!(options, rule.options);
    }
}

/// `filename code line column` for every record,
///  in output order.
fn located(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::<String>::new();
    for record in records(text) {
        found.push(format!(
            "{} {} {} {}",
            record["filename"].as_str().expect("filename"),
            record["code"].as_str().expect("code"),
            record["labels"][0]["span"]["line"],
            record["labels"][0]["span"]["column"]
        ));
    }
    return found;
}

/// The sorted names of a directory's entries.
fn entries(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = Vec::<String>::new();
    for entry in std::fs::read_dir(directory).expect("readable directory") {
        names.push(
            entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned(),
        );
    }
    names.sort();
    return names;
}

/// Help and version are parser outcomes with status 0;
///  usage errors are status 2 with nothing on standard output.
#[test]
fn help_version_and_usage_errors_use_the_documented_statuses() {
    let fixture: Fixture = Fixture::new();
    let help: Run = run(&fixture.path, &["--help"], b"");
    assert_eq!(help.status, 0);
    assert!(
        help.stdout
            .starts_with(format!("{ABOUT}\n\nUsage: {PROGRAM} [OPTIONS] [PATH]...").as_str()),
        "{}",
        help.stdout
    );
    for option in long_options() {
        assert!(
            help.stdout.contains(&option),
            "{option} is missing from:\n{}",
            help.stdout
        );
    }
    assert_eq!(help.stderr, "");
    // Read the version from the manifest so a release bump cannot leave this assertion stale.
    let expected_version: String = format!("{PROGRAM} {}\n", env!("CARGO_PKG_VERSION"));
    let version: Run = run(&fixture.path, &["--version"], b"");
    assert_eq!(
        (
            version.status,
            version.stdout.as_str(),
            version.stderr.as_str()
        ),
        (0, expected_version.as_str(), "")
    );
    for arguments in [
        vec!["--bogus"],
        vec!["--stdin"],
        vec!["--stdin-filename", "a.md"],
        vec!["--stdin", "--stdin-filename", "a.md", "extra.md"],
        vec!["--concurrency", "0"],
        vec!["--max-warnings", "-1"],
        vec!["--rule", "markdown/single-h1"],
        vec!["--fix", "--rules"],
        vec!["--fix-content", "--rules"],
        vec!["--init", "--rules"],
    ] {
        let usage: Run = run(&fixture.path, arguments.as_slice(), b"");
        assert_eq!(usage.status, 2, "{arguments:?}");
        assert_eq!(usage.stdout, "", "{arguments:?}");
        assert!(
            usage.stderr.starts_with("error: "),
            "{arguments:?}: {}",
            usage.stderr
        );
    }
    assert!(entries(&fixture.path).is_empty());
}

/// Status 0 prints nothing,
///  status 1 prints JSONL on standard output only,
///  status 2 explains itself on standard error.
#[test]
fn exit_statuses_and_streams_follow_the_contract() {
    let fixture: Fixture = Fixture::new();
    fixture.write("a.md", "# Title.\n");
    let unconfigured: Run = run(&fixture.path, &[], b"");
    assert_eq!((unconfigured.status, unconfigured.stdout.as_str()), (2, ""));
    assert!(
        unconfigured
            .stderr
            .starts_with(format!("{PROGRAM}: No {CONFIG_NAME} was found").as_str()),
        "{}",
        unconfigured.stderr
    );
    assert_eq!(unconfigured.stderr.lines().count(), 1);
    fixture.write(CONFIG, RULES);
    let findings: Run = run(&fixture.path, &[], b"");
    assert_eq!((findings.status, findings.stderr.as_str()), (1, ""));
    assert_eq!(
        findings.stdout,
        "{\"message\":\"Heading ends with punctuation; remove the trailing punctuation.\",\"code\":\"markdown/no-trailing-punctuation\",\"severity\":\"error\",\"causes\":[],\"filename\":\"a.md\",\"labels\":[{\"span\":{\"offset\":0,\"length\":8,\"line\":1,\"column\":1}}],\"related\":[]}\n"
    );
    fixture.write("a.md", "# Title\n");
    let clean: Run = run(&fixture.path, &[], b"");
    assert_eq!(
        (clean.status, clean.stdout.as_str(), clean.stderr.as_str()),
        (0, "", "")
    );
    fixture.write("broken.mdx", "<A>\ntext\n</B>\n");
    let incomplete: Run = run(&fixture.path, &[], b"");
    assert_eq!((incomplete.status, incomplete.stderr.as_str()), (2, ""));
    assert_eq!(located(incomplete.stdout.as_str()).len(), 1);
    assert_eq!(
        records(incomplete.stdout.as_str())[0]["code"],
        PROCESSING_FAILURE
    );
    assert_eq!(
        records(incomplete.stdout.as_str())[0]["filename"],
        "broken.mdx"
    );
    fixture.write(CONFIG, "[{ \"files\": 3 }]");
    let malformed: Run = run(&fixture.path, &["--fix-content"], b"");
    assert_eq!((malformed.status, malformed.stdout.as_str()), (2, ""));
    assert!(
        malformed
            .stderr
            .starts_with(format!("{PROGRAM}: Configuration ").as_str()),
        "{}",
        malformed.stderr
    );
    assert!(malformed.stderr.contains(CONFIG), "{}", malformed.stderr);
    let missing: Run = run(&fixture.path, &["absent.md"], b"");
    assert_eq!(missing.status, 2);
    let warned: Fixture = Fixture::new();
    warned.write(CONFIG, r#"[{ "files": ["**/*.md"], "rules": { "markdown/no-trailing-punctuation": { "severity": "warn" } } }]"#);
    warned.write("a.md", "# Title.\n");
    assert_eq!(run(&warned.path, &[], b"").status, 0);
    assert_eq!(run(&warned.path, &["--max-warnings", "0"], b"").status, 1);
    let quiet: Run = run(&warned.path, &["--max-warnings", "0", "--quiet"], b"");
    assert_eq!(
        (quiet.status, quiet.stdout.as_str(), quiet.stderr.as_str()),
        (1, "", "")
    );
}

/// The walker reads ignore files,
///  skips Git metadata and `node_modules`,
///  enters hidden directories and picks only supported extensions.
#[test]
fn walking_selects_supported_files_and_honours_ignores() {
    let fixture: Fixture = Fixture::new();
    fixture.write(CONFIG, RULES);
    std::fs::create_dir(fixture.path.join(".git")).expect("repository marker");
    fixture.write(".git/info/exclude", "excluded-by-info/\n");
    fixture.write(".git/a.md", "# Title.\n");
    fixture.write(".gitignore", "ignored/\n*.generated.md\n");
    fixture.write("nested/.gitignore", "local-only.md\n");
    fixture.write(".ignore", "dot-ignored/\n");
    for file in [
        "a.md",
        "b.mdx",
        "c.rs",
        "d.txt",
        "e.MD",
        ".hidden/a.md",
        "ignored/a.md",
        "dot-ignored/a.md",
        "excluded-by-info/a.md",
        "node_modules/pkg/a.md",
        "nested/a.md",
        "nested/local-only.md",
        "nested/deep/x.generated.md",
        "extra/a.md",
    ] {
        fixture.write(file, "# Title.\n");
    }
    fixture.write(
        "c.rs",
        "//! File.\n\n/// Item.\nfn item() {}\n\nfn undocumented() {}\n",
    );
    let default: Run = run(&fixture.path, &[], b"");
    assert_eq!((default.status, default.stderr.as_str()), (1, ""));
    assert_eq!(
        located(default.stdout.as_str()),
        [
            ".hidden/a.md markdown/no-trailing-punctuation 1 1",
            "a.md markdown/no-trailing-punctuation 1 1",
            "b.mdx markdown/no-trailing-punctuation 1 1",
            "c.rs rust/require-rustdoc 6 1",
            "extra/a.md markdown/no-trailing-punctuation 1 1",
            "nested/a.md markdown/no-trailing-punctuation 1 1",
        ]
    );
    fixture.write("more.ignore", "extra/\n");
    let flagged: Run = run(
        &fixture.path,
        &[
            "--ignore-path",
            "more.ignore",
            "--ignore-pattern",
            "nested/**",
            "--ignore-pattern",
            "*.mdx",
        ],
        b"",
    );
    assert_eq!(
        located(flagged.stdout.as_str()),
        [
            ".hidden/a.md markdown/no-trailing-punctuation 1 1",
            "a.md markdown/no-trailing-punctuation 1 1",
            "c.rs rust/require-rustdoc 6 1",
        ]
    );
    let unignored: Run = run(&fixture.path, &["--no-ignore", "--quiet"], b"");
    assert_eq!(records(unignored.stdout.as_str()).len(), 12);
    // Path arguments: a directory is walked, a named file bypasses traversal ignores, a glob is expanded.
    let named: Run = run(&fixture.path, &["nested", "ignored/a.md", "ex*/*.md"], b"");
    assert_eq!(
        located(named.stdout.as_str()),
        [
            "extra/a.md markdown/no-trailing-punctuation 1 1",
            "ignored/a.md markdown/no-trailing-punctuation 1 1",
            "nested/a.md markdown/no-trailing-punctuation 1 1",
        ]
    );
    let unmatched: Run = run(&fixture.path, &["no-such-*/*.md"], b"");
    assert_eq!((unmatched.status, unmatched.stdout.as_str()), (2, ""));
    let allowed: Run = run(
        &fixture.path,
        &["--no-error-on-unmatched-pattern", "no-such-*/*.md"],
        b"",
    );
    assert_eq!(
        (
            allowed.status,
            allowed.stdout.as_str(),
            allowed.stderr.as_str()
        ),
        (0, "", "")
    );
}

/// The nearest configuration is used alone,
///  `--config` resolves against the working directory,
///  and blocks merge in order.
#[test]
fn configuration_lookup_and_merging_follow_the_design() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        CONFIG,
        r#"// Comments and trailing commas are accepted.
[
  { "ignores": ["vendor/"], },
  { "files": ["**/*.rs"], "rules": { "rust/max-lines": { "severity": "error", "max": 2 }, "rust/require-rustdoc": { "severity": "error" } } },
  { "files": ["src/**/*.rs"], "rules": { "rust/max-lines": { "severity": "warn" } } },
  { "files": ["**/*.md"], "rules": { "markdown/lfs-image-url": { "severity": "off", "exclude": ["a/"] } } },
  { "files": ["doc/**"], "rules": { "markdown/lfs-image-url": { "severity": "off", "exclude": ["b/"] } } },
]"#,
    );
    fixture.write(
        &format!("nested/{CONFIG}"),
        r#"[{ "files": ["*.rs"], "rules": { "rust/tests-in-sidecars": { "severity": "warn" } } }]"#,
    );
    // Each file holds one documented stray unit test on its eleventh line, which only the sidecar rule reports.
    let three_lines: &str = "//! File.\n\n/// One.\nfn one() {}\n/// Two.\nfn two() {}\n/// Three.\nfn three() {}\n/// Stray.\n#[test]\nfn stray() {}\n";
    for file in [
        "src/a.rs",
        "top.rs",
        "vendor/a.rs",
        "nested/a.rs",
        "nested/deep/a.rs",
    ] {
        fixture.write(file, three_lines);
    }
    let output: Run = run(&fixture.path, &[], b"");
    assert_eq!((output.status, output.stderr.as_str()), (1, ""));
    let lines: Vec<serde_json::Value> = records(output.stdout.as_str());
    let mut summary: Vec<String> = Vec::<String>::new();
    for line in &lines {
        summary.push(format!(
            "{} {} {}",
            line["filename"].as_str().expect("filename"),
            line["code"].as_str().expect("code"),
            line["severity"].as_str().expect("severity")
        ));
    }
    // The nested configuration governs its subtree alone: no rustdoc or line budget there, and its
    // single-star pattern does not reach `nested/deep`. The later `src` block changes only the severity.
    // The outer configuration has no sidecar rule, so the stray test is reported only below `nested`.
    assert_eq!(
        summary,
        [
            "nested/a.rs rust/tests-in-sidecars warn",
            "src/a.rs rust/max-lines warn",
            "top.rs rust/max-lines error",
        ]
    );
    assert!(
        lines[1]["message"]
            .as_str()
            .expect("message")
            .contains("limit is 2"),
        "{}",
        lines[1]["message"]
    );
    // Arrays from every applying block concatenate; scalars take the last value.
    let printed: Run = run(&fixture.path, &["--print-config", "doc/a.md"], b"");
    assert_eq!((printed.status, printed.stderr.as_str()), (0, ""));
    let compact: String = printed
        .stdout
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("");
    assert!(
        compact.contains(r#""state":"configured""#),
        "{}",
        printed.stdout
    );
    assert!(
        compact.contains(r#""exclude":["a/","b/"]"#),
        "{}",
        printed.stdout
    );
    let merged: Run = run(&fixture.path, &["--print-config", "src/a.rs"], b"");
    let merged_compact: String = merged
        .stdout
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("");
    assert!(
        merged_compact.contains(r#""rust/max-lines":{"severity":"warn","max":2}"#),
        "{}",
        merged.stdout
    );
    for (file, state) in [
        ("vendor/a.rs", "ignored"),
        ("nested/deep/a.rs", "unconfigured"),
        ("a.txt", "unconfigured"),
    ] {
        let state_output: Run = run(&fixture.path, &["--print-config", file], b"");
        assert!(
            state_output
                .stdout
                .contains(format!("\"{state}\"").as_str()),
            "{file}: {}",
            state_output.stdout
        );
    }
    // An explicit configuration outside the tree replaces lookup and matches working-directory paths.
    let outside: Fixture = Fixture::new();
    let explicit: PathBuf = outside.write("only.jsonc", r#"[{ "files": ["nested/**/*.rs"], "rules": { "rust/tests-in-sidecars": { "severity": "error" } } }]"#);
    let explicit_text: String = explicit.to_string_lossy().into_owned();
    let overridden: Run = run(&fixture.path, &["--config", explicit_text.as_str()], b"");
    assert_eq!(
        located(overridden.stdout.as_str()),
        [
            "nested/a.rs rust/tests-in-sidecars 11 4",
            "nested/deep/a.rs rust/tests-in-sidecars 11 4",
        ]
    );
    // Running from a subdirectory still finds the ancestor configuration and names files relative to that directory.
    let below: Run = run(&fixture.path.join("src"), &[], b"");
    // The finding sits on the first code line beyond the budget of two.
    assert_eq!(located(below.stdout.as_str()), ["a.rs rust/max-lines 8 1"]);
}

/// A configuration that still selects the removed anonymous-function rule is a setup error on status 2,
///  naming the rule,
///  and lints nothing;
///  the same file without that rule runs normally.
#[test]
fn a_configuration_naming_the_removed_anonymous_function_rule_is_a_setup_error() {
    let fixture: Fixture = Fixture::new();
    fixture.write("a.rs", "fn main() { call(|| 1); }\n");
    fixture.write(
        CONFIG,
        r#"[{ "files": ["**/*.rs"], "rules": { "rust/no-anonymous-functions": { "severity": "off" } } }]"#,
    );
    let stale: Run = run(&fixture.path, &[], b"");
    assert_eq!((stale.status, stale.stdout.as_str()), (2, ""));
    assert!(
        stale
            .stderr
            .starts_with(format!("{PROGRAM}: Configuration ").as_str()),
        "{}",
        stale.stderr
    );
    assert!(
        stale
            .stderr
            .contains("Unknown built-in rule rust/no-anonymous-functions"),
        "{}",
        stale.stderr
    );
    fixture.write(
        CONFIG,
        r#"[{ "files": ["**/*.rs"], "rules": { "rust/max-lines": { "severity": "error", "max": 300 } } }]"#,
    );
    let current: Run = run(&fixture.path, &[], b"");
    assert_eq!(
        (
            current.status,
            current.stdout.as_str(),
            current.stderr.as_str()
        ),
        (0, "", "")
    );
}

/// Every configuration rejection diagnostic names the running binary,
///  so which binary refused the configuration is readable from the failure itself.
/// A lockfile pinning exactly the running version measures no staleness,
///  so no hint appears.
#[test]
fn configuration_rejection_diagnostics_identify_the_running_binary() {
    // Read the version from the manifest so a release bump cannot leave this assertion stale.
    let identity: String = format!("({PROGRAM} {})", env!("CARGO_PKG_VERSION"));
    let fixture: Fixture = Fixture::new();
    fixture.write("a.rs", "fn main() {}\n");
    fixture.write(
        LOCK_NAME,
        format!(
            "{TOOL_TABLE}\nversion = \"{}\"\n",
            env!("CARGO_PKG_VERSION")
        )
        .as_str(),
    );
    fixture.write(
        CONFIG,
        r#"[{ "files": ["**/*.rs"], "rules": { "rust/no-anonymous-functions": { "severity": "off" } } }]"#,
    );
    let unknown: Run = run(&fixture.path, &[], b"");
    assert_eq!((unknown.status, unknown.stdout.as_str()), (2, ""));
    assert!(
        unknown
            .stderr
            .contains("Unknown built-in rule rust/no-anonymous-functions"),
        "{}",
        unknown.stderr
    );
    assert!(
        unknown.stderr.contains(identity.as_str()),
        "{}",
        unknown.stderr
    );
    assert!(
        !unknown.stderr.contains("may be stale"),
        "{}",
        unknown.stderr
    );
    fixture.write(
        CONFIG,
        r#"[{ "files": ["**/*.rs"], "rules": { "rust/max-lines": { "severity": "warning" } } }]"#,
    );
    let severity: Run = run(&fixture.path, &[], b"");
    assert_eq!((severity.status, severity.stdout.as_str()), (2, ""));
    assert!(
        severity.stderr.contains("severity must be"),
        "{}",
        severity.stderr
    );
    assert!(
        severity.stderr.contains(identity.as_str()),
        "{}",
        severity.stderr
    );
    fixture.write(CONFIG, "[{");
    let malformed: Run = run(&fixture.path, &[], b"");
    assert_eq!((malformed.status, malformed.stdout.as_str()), (2, ""));
    assert!(
        malformed.stderr.contains("JSONC syntax error"),
        "{}",
        malformed.stderr
    );
    assert!(
        malformed.stderr.contains(identity.as_str()),
        "{}",
        malformed.stderr
    );
}

/// The unknown-rule error names a stale resolved binary only when the repository's lockfile
///  measures a pin newer than the running binary,
///  and the hint quotes that measurement.
#[test]
fn a_newer_repository_pin_adds_the_stale_binary_hint() {
    let major: u64 = env!("CARGO_PKG_VERSION")
        .split('.')
        .next()
        .expect("manifest version has a major component")
        .parse()
        .expect("major component is numeric");
    // One major version ahead of this build, so a release bump cannot make the control stale.
    // A release pin and a pre-release pin, both one major version ahead of this build, which is itself a
    // pre-release while the package is in its alpha series.
    for pin in [
        format!("{}.0.0", major + 1),
        format!("{}.0.0-alpha.1", major + 1),
    ] {
        let fixture: Fixture = Fixture::new();
        fixture.write("a.rs", "fn main() {}\n");
        fixture.write(
            LOCK_NAME,
            format!("{TOOL_TABLE}\nversion = \"{pin}\"\n").as_str(),
        );
        fixture.write(
            CONFIG,
            r#"[{ "files": ["**/*.rs"], "rules": { "rust/no-anonymous-functions": { "severity": "off" } } }]"#,
        );
        let tool_id: &str = TOOL_TABLE
            .strip_prefix("[[tools.\"")
            .and_then(|rest| return rest.strip_suffix("\"]]"))
            .expect("the tool table header wraps the tool identity");
        let stale: Run = run(&fixture.path, &[], b"");
        assert_eq!((stale.status, stale.stdout.as_str()), (2, ""));
        assert_eq!(
            stale.stderr,
            format!(
                "{PROGRAM}: Configuration {0}/{1}: Unknown built-in rule rust/no-anonymous-functions. ({PROGRAM} {2}) The resolved binary may be stale: {0}/{3} pins {4} {pin}, newer than this binary.\n",
                fixture.path.display(),
                CONFIG,
                env!("CARGO_PKG_VERSION"),
                LOCK_NAME,
                tool_id
            )
        );
    }
}

/// A repository pin older than the running binary measures no staleness,
///  so the hint stays out
/// even though the rule is still unknown.
#[test]
fn an_older_repository_pin_adds_no_stale_binary_hint() {
    assert_ne!(
        env!("CARGO_PKG_VERSION"),
        "0.0.0",
        "the control needs a strictly older pin"
    );
    let fixture: Fixture = Fixture::new();
    fixture.write("a.rs", "fn main() {}\n");
    fixture.write(
        LOCK_NAME,
        format!("{TOOL_TABLE}\nversion = \"0.0.0\"\n").as_str(),
    );
    fixture.write(
        CONFIG,
        r#"[{ "files": ["**/*.rs"], "rules": { "rust/no-anonymous-functions": { "severity": "off" } } }]"#,
    );
    let resolved: Run = run(&fixture.path, &[], b"");
    assert_eq!((resolved.status, resolved.stdout.as_str()), (2, ""));
    assert!(
        resolved
            .stderr
            .contains("Unknown built-in rule rust/no-anonymous-functions"),
        "{}",
        resolved.stderr
    );
    assert!(
        !resolved.stderr.contains("may be stale"),
        "{}",
        resolved.stderr
    );
}

/// `--fix` replaces files atomically,
///  keeps their permission bits,
///  leaves no temporary files,
///  and reports the remainder.
#[cfg(unix)]
#[test]
fn fixing_writes_atomically_and_keeps_the_file_mode() {
    use std::os::unix::fs::{
        MetadataExt,
        PermissionsExt,
    };
    let fixture: Fixture = Fixture::new();
    fixture.write(CONFIG, RULES);
    let fixable: PathBuf = fixture.write("doc/a.md", "# Title.\n\n```\ncode\n```\n\n# Second\n");
    let clean: PathBuf = fixture.write("doc/clean.md", "# Title\n");
    let unfixable: PathBuf = fixture.write("doc/b.md", "# One\n\n### Three\n");
    std::fs::set_permissions(&fixable, std::fs::Permissions::from_mode(0o640)).expect("chmod");
    std::fs::set_permissions(&clean, std::fs::Permissions::from_mode(0o444)).expect("chmod");
    std::fs::set_permissions(&unfixable, std::fs::Permissions::from_mode(0o600)).expect("chmod");
    let before_fixable: std::fs::Metadata = std::fs::metadata(&fixable).expect("metadata");
    let before_clean: std::fs::Metadata = std::fs::metadata(&clean).expect("metadata");
    let before_unfixable: std::fs::Metadata = std::fs::metadata(&unfixable).expect("metadata");
    let output: Run = run(&fixture.path, &["--fix-content"], b"");
    assert_eq!((output.status, output.stderr.as_str()), (1, ""));
    assert_eq!(
        located(output.stdout.as_str()),
        [
            "doc/a.md markdown/single-h1 7 1",
            "doc/b.md markdown/heading-increment 3 1",
        ]
    );
    assert_eq!(
        fixture.read("doc/a.md"),
        "# Title\n\n```text\ncode\n```\n\n# Second\n"
    );
    let after_fixable: std::fs::Metadata = std::fs::metadata(&fixable).expect("metadata");
    assert_eq!(after_fixable.permissions().mode() & 0o7777, 0o640);
    // A rename replaced the file: it is a different inode, never a truncated original.
    assert_ne!(after_fixable.ino(), before_fixable.ino());
    // Files with nothing to fix are not rewritten at all: same inode, same modification time, same mode.
    for (path, before) in [(&clean, &before_clean), (&unfixable, &before_unfixable)] {
        let after: std::fs::Metadata = std::fs::metadata(path).expect("metadata");
        assert_eq!(after.ino(), before.ino());
        assert_eq!(after.mtime_nsec(), before.mtime_nsec());
        assert_eq!(after.mtime(), before.mtime());
        assert_eq!(after.permissions().mode(), before.permissions().mode());
    }
    assert_eq!(fixture.read("doc/clean.md"), "# Title\n");
    assert_eq!(
        entries(&fixture.path.join("doc")),
        ["a.md", "b.md", "clean.md"]
    );
    // A second run is a fixed point: nothing is rewritten and the same findings remain.
    let settled: std::fs::Metadata = std::fs::metadata(&fixable).expect("metadata");
    let again: Run = run(&fixture.path, &["--fix-content"], b"");
    assert_eq!(again.stdout, output.stdout);
    assert_eq!(
        std::fs::metadata(&fixable).expect("metadata").ino(),
        settled.ino()
    );
    // A symbolic link is followed: the link stays a link and its target is fixed.
    let target: PathBuf = fixture.write("real/target.md", "# Linked.\n");
    std::os::unix::fs::symlink(&target, fixture.path.join("link.md")).expect("symlink");
    let linked: Run = run(&fixture.path, &["--fix-content", "link.md"], b"");
    assert_eq!((linked.status, linked.stdout.as_str()), (0, ""));
    assert!(
        std::fs::symlink_metadata(fixture.path.join("link.md"))
            .expect("link")
            .file_type()
            .is_symlink()
    );
    assert_eq!(fixture.read("real/target.md"), "# Linked\n");
    // A refused fix leaves the file's bytes and inode alone and exits 2.
    let refusal: Fixture = Fixture::new();
    refusal.write(CONFIG, r#"[{ "files": ["**/*.md"], "rules": { "markdown/link-image-reference-definitions": { "severity": "error" } } }]"#);
    let only_definition: PathBuf = refusal.write("a.md", "[unused]: https://example.com\n");
    let before_refusal: u64 = std::fs::metadata(&only_definition).expect("metadata").ino();
    let refused: Run = run(&refusal.path, &["--fix-content"], b"");
    assert_eq!((refused.status, refused.stderr.as_str()), (2, ""));
    let mut refused_codes: Vec<String> = Vec::<String>::new();
    for record in records(refused.stdout.as_str()) {
        refused_codes.push(String::from(record["code"].as_str().expect("code")));
    }
    assert_eq!(
        refused_codes,
        [FIX_REFUSED, "markdown/link-image-reference-definitions"]
    );
    assert_eq!(refusal.read("a.md"), "[unused]: https://example.com\n");
    assert_eq!(
        std::fs::metadata(&only_definition).expect("metadata").ino(),
        before_refusal
    );
    assert_eq!(entries(&refusal.path), ["a.md", CONFIG]);
}

/// Processor findings report the host file and host positions,
///  and a rustdoc fix is written into the host comment.
#[test]
fn processors_report_and_fix_at_host_positions() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        CONFIG,
        r#"[
          { "files": ["**/*.rs"], "ignores": ["**/*.md/**", "**/*.mdx/**"], "rules": { "rust/require-rustdoc": { "severity": "error" } } },
          { "files": ["**/*.rs/*.md"], "rules": { "markdown/fenced-code-language": { "severity": "error" }, "markdown/no-trailing-punctuation": { "severity": "error" } } },
          { "files": ["**/*.md/*.rs", "**/*.mdx/*.rs", "**/*.rs/*.md/*.rs"], "rules": { "rust/require-rustdoc": { "severity": "warn" } } }
        ]"#,
    );
    let markdown: &str = "Intro 🚀.\r\n\r\n> ```rust,no_run\r\n> # let hidden = 1;\r\n> let value = \"🚀\"; fn helper() {}\r\n> ```\r\n";
    let rust: &str = "//! File.\n\n    /// # Heading.\n    ///\n    /// ```\n    /// //! Example.\n    /// let value = 1; fn helper() {}\n    /// ```\n    fn item() {}\n";
    fixture.write("doc/a.md", markdown);
    fixture.write(
        "doc/b.mdx",
        "<section>\n\n```rs\n//! Embedded.\nlet value = 2; fn helper() {}\n```\n\n</section>\n",
    );
    fixture.write("src/c.rs", rust);
    let output: Run = run(&fixture.path, &[], b"");
    assert_eq!((output.status, output.stderr.as_str()), (1, ""));
    assert_eq!(
        located(output.stdout.as_str()),
        [
            "doc/a.md rust/require-rustdoc 4 5",
            "doc/a.md rust/require-rustdoc 5 21",
            "doc/b.mdx rust/require-rustdoc 5 16",
            "src/c.rs markdown/no-trailing-punctuation 3 9",
            "src/c.rs markdown/fenced-code-language 5 9",
            "src/c.rs rust/require-rustdoc 7 24",
        ]
    );
    // Byte ranges index the host file exactly, across CRLF, a quote prefix and astral characters.
    let lines: Vec<serde_json::Value> = records(output.stdout.as_str());
    let markdown_item: &serde_json::Value = &lines[1]["labels"][0]["span"];
    let offset: usize = markdown_item["offset"].as_u64().expect("offset") as usize;
    let length: usize = markdown_item["length"].as_u64().expect("length") as usize;
    assert_eq!(&markdown[offset..offset + length], "fn helper() {}");
    let rust_item: &serde_json::Value = &lines[5]["labels"][0]["span"];
    let rust_offset: usize = rust_item["offset"].as_u64().expect("offset") as usize;
    let rust_length: usize = rust_item["length"].as_u64().expect("length") as usize;
    assert_eq!(
        &rust[rust_offset..rust_offset + rust_length],
        "fn helper() {}"
    );
    let heading: usize = lines[3]["labels"][0]["span"]["offset"]
        .as_u64()
        .expect("offset") as usize;
    assert!(
        rust[heading..].starts_with("# Heading."),
        "{}",
        &rust[heading..]
    );
    // Fixing rewrites only the host comment lines, keeping their indentation and prefix.
    let fixed: Run = run(&fixture.path, &["--fix-content"], b"");
    assert_eq!(fixed.status, 0);
    assert_eq!(
        fixture.read("src/c.rs"),
        "//! File.\n\n    /// # Heading\n    ///\n    /// ```rust\n    /// //! Example.\n    /// let value = 1; fn helper() {}\n    /// ```\n    fn item() {}\n"
    );
    assert_eq!(fixture.read("doc/a.md"), markdown);
    assert_eq!(
        located(fixed.stdout.as_str()),
        [
            "doc/a.md rust/require-rustdoc 4 5",
            "doc/a.md rust/require-rustdoc 5 21",
            "doc/b.mdx rust/require-rustdoc 5 16",
            "src/c.rs rust/require-rustdoc 7 24",
        ]
    );
}

/// Standard-input fixing prints exactly the fixed source on standard output and only JSONL on standard error.
#[test]
fn standard_input_fixing_separates_source_from_findings() {
    let fixture: Fixture = Fixture::new();
    fixture.write(CONFIG, RULES);
    let source: &str = "\u{feff}# Title.\r\n\r\n```\r\ncode 🚀\r\n```\r\n\r\n# Second\r\n";
    let fixed: Run = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "doc/new.md", "--fix-content"],
        source.as_bytes(),
    );
    assert_eq!(fixed.status, 1);
    assert_eq!(
        fixed.stdout,
        "\u{feff}# Title\r\n\r\n```text\r\ncode 🚀\r\n```\r\n\r\n# Second\r\n"
    );
    assert_eq!(
        located(fixed.stderr.as_str()),
        ["doc/new.md markdown/single-h1 7 1"]
    );
    assert!(!fixture.path.join("doc").exists());
    let lint: Run = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "doc/new.md"],
        source.as_bytes(),
    );
    assert_eq!((lint.status, lint.stderr.as_str()), (1, ""));
    assert_eq!(
        located(lint.stdout.as_str()),
        [
            "doc/new.md markdown/no-trailing-punctuation 1 1",
            "doc/new.md markdown/fenced-code-language 3 1",
            "doc/new.md markdown/single-h1 7 1",
        ]
    );
    let clean: Run = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "a.rs", "--fix-content"],
        b"//! File.\n",
    );
    assert_eq!(
        (clean.status, clean.stdout.as_str(), clean.stderr.as_str()),
        (0, "//! File.\n", "")
    );
    let silent: Run = run(
        &fixture.path,
        &[
            "--stdin",
            "--stdin-filename",
            "a.md",
            "--fix-content",
            "--silent",
        ],
        b"# Title.\n\n# Second\n",
    );
    assert_eq!(
        (
            silent.status,
            silent.stdout.as_str(),
            silent.stderr.as_str()
        ),
        (1, "# Title\n\n# Second\n", "")
    );
    // Setup failures print no source: a consumer must not write standard output on a non-zero status.
    let invalid: Run = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "a.md", "--fix-content"],
        &[b'#', b' ', 0xff, b'\n'],
    );
    assert_eq!((invalid.status, invalid.stdout.as_str()), (2, ""));
    assert!(
        invalid
            .stderr
            .starts_with(format!("{PROGRAM}: Standard input is not valid UTF-8").as_str()),
        "{}",
        invalid.stderr
    );
    let unsupported: Run = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "a.txt", "--fix-content"],
        b"x",
    );
    assert_eq!((unsupported.status, unsupported.stdout.as_str()), (2, ""));
    // An MDX error under --fix returns the original source with the failure on standard error.
    let broken: Run = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "a.mdx", "--fix-content"],
        b"# Title.\n\n<A>\ntext\n</B>\n",
    );
    assert_eq!(broken.status, 2);
    assert_eq!(broken.stdout, "# Title.\n\n<A>\ntext\n</B>\n");
    let mut broken_codes: Vec<String> = Vec::<String>::new();
    for record in records(broken.stderr.as_str()) {
        broken_codes.push(String::from(record["code"].as_str().expect("code")));
    }
    assert_eq!(broken_codes, [FIX_REFUSED, PROCESSING_FAILURE]);
}

/// An LFS-tracked image over the rule's bound is refused with the option that raises it,
///  and a configuration that raises the option reads the image,
///  so a repository with a large image needs no edit to the linter's source.
#[test]
fn a_large_lfs_image_is_refused_until_configuration_raises_the_bound() {
    let repository: Fixture = Fixture::new();
    repository.write(".lfsconfig", "[lfs]\n\turl = https://lfs.example\n");
    repository.write(
        ".gitattributes",
        "*.png filter=lfs diff=lfs merge=lfs -text\n",
    );
    // A sparse image one byte over the default bound: the bound is what the test needs, not the bytes.
    let image: PathBuf = repository.write("pkg/asset/huge.png", "");
    let file: std::fs::File = std::fs::File::create(&image).expect("fixture image");
    file.set_len(DEFAULT_MAX_IMAGE_BYTES + 1)
        .expect("sparse image one byte over the default bound");
    drop(file);
    let document: &[u8] = b"# Title.\n\n![huge](asset/huge.png)\n";
    let arguments: [&str; 4] = [
        "--stdin",
        "--stdin-filename",
        "pkg/README.md",
        "--fix-content",
    ];
    repository.write(
        CONFIG,
        r#"[{ "files": ["**/*.md"], "rules": { "markdown/lfs-image-url": { "severity": "error" } } }]"#,
    );
    // The default bound refuses the image, and the message names the option rather than a source constant.
    let refused: Run = run(&repository.path, &arguments, document);
    assert_eq!(refused.status, 2, "{}", refused.stderr);
    assert!(
        refused.stderr.contains("max-image-bytes"),
        "{}",
        refused.stderr
    );
    assert!(
        !refused.stderr.contains("src/markdown"),
        "the refusal must not send the reader into the source: {}",
        refused.stderr
    );
    // Raising the option in the configuration reads the image and rewrites its link.
    let raised_bound: u64 = DEFAULT_MAX_IMAGE_BYTES * 2;
    repository.write(
        CONFIG,
        format!(
            r#"[{{ "files": ["**/*.md"], "rules": {{ "markdown/lfs-image-url": {{ "severity": "error", "max-image-bytes": {raised_bound} }} }} }}]"#
        )
        .as_str(),
    );
    let raised: Run = run(&repository.path, &arguments, document);
    assert_eq!(raised.status, 0, "{}", raised.stderr);
    assert!(
        raised.stdout.contains("/pkg/asset/huge.png"),
        "{}",
        raised.stdout
    );
}

/// The largest value a platform's word can hold is refused when the configuration loads,
///  naming the option and the largest bound the reader can probe past,
///  and the largest bound it does take reads the image and rewrites its link.
/// Why the boundary is here:
///  the rule reads one byte past the bound, so a bound at the platform's maximum
///  wraps to zero in a build without overflow checks, reads no byte and hashes empty input.
#[test]
fn an_image_bound_the_probe_cannot_pass_is_refused_at_load() {
    let repository: Fixture = Fixture::new();
    repository.write(".lfsconfig", "[lfs]\n\turl = https://lfs.example\n");
    repository.write(
        ".gitattributes",
        "*.png filter=lfs diff=lfs merge=lfs -text\n",
    );
    repository.write("pkg/asset/shot.png", "image bytes");
    let document: &[u8] = b"# Title.\n\n![shot](asset/shot.png)\n";
    let arguments: [&str; 4] = [
        "--stdin",
        "--stdin-filename",
        "pkg/README.md",
        "--fix-content",
    ];
    // The bound with no answer past it: reading one byte more would wrap the reader's counter.
    let unprobeable: u64 = MAX_IMAGE_BYTES_BOUND + 1;
    repository.write(
        CONFIG,
        format!(
            r#"[{{ "files": ["**/*.md"], "rules": {{ "markdown/lfs-image-url": {{ "severity": "error", "max-image-bytes": {unprobeable} }} }} }}]"#
        )
        .as_str(),
    );
    let refused: Run = run(&repository.path, &arguments, document);
    assert_eq!(refused.status, 2, "{}", refused.stderr);
    assert_eq!(refused.stdout, "", "{}", refused.stderr);
    assert!(
        refused.stderr.contains("max-image-bytes"),
        "{}",
        refused.stderr
    );
    // The largest value the option takes is the one the reader's bound names, derived rather than copied.
    let largest: String = MAX_IMAGE_BYTES_BOUND.to_string();
    assert!(
        refused.stderr.contains(largest.as_str()),
        "the refusal must name the largest value it takes: {}",
        refused.stderr
    );
    assert!(
        !refused.stderr.contains("panicked"),
        "the bound is refused, never read under arithmetic that cannot hold it: {}",
        refused.stderr
    );
    // The control: that largest value itself is accepted, reads the image and rewrites its link.
    repository.write(
        CONFIG,
        format!(
            r#"[{{ "files": ["**/*.md"], "rules": {{ "markdown/lfs-image-url": {{ "severity": "error", "max-image-bytes": {largest} }} }} }}]"#
        )
        .as_str(),
    );
    let accepted: Run = run(&repository.path, &arguments, document);
    assert_eq!(
        (accepted.status, accepted.stderr.as_str()),
        (0, ""),
        "{}",
        accepted.stderr
    );
    // Measured with coreutils `sha256sum` for the bytes `image bytes`.
    let oid: &str = "de7030234493a8bea844dbe1d8676e68a2c1a4b014c721f0425a22b6df66faec";
    assert!(
        accepted.stdout.contains(oid),
        "the largest bound reads the real bytes: {}",
        accepted.stdout
    );
}

/// The commit-time shape:
///  a temporary one-rule configuration outside the repository,
///  standard input,
///  and fixing.
#[test]
fn the_commit_adapter_invocation_rewrites_lfs_images() {
    let repository: Fixture = Fixture::new();
    repository.write(
        ".lfsconfig",
        "[lfs]\n\turl = https://lfs:token@lfs.example\n",
    );
    repository.write(
        ".gitattributes",
        "*.png filter=lfs diff=lfs merge=lfs -text\n",
    );
    repository.write("pkg/asset/shot.png", "image bytes");
    repository.write("pkg/asset/plain.svg", "<svg/>");
    // A discovered configuration that would turn the rule off must be skipped by --config.
    repository.write(CONFIG, "[]");
    let temporary: Fixture = Fixture::new();
    let config: PathBuf = temporary.write(
        "one-rule.jsonc",
        r#"[{ "files": ["**/*.md", "**/*.mdx"], "rules": { "markdown/lfs-image-url": { "severity": "error", "exclude": ["package/ssg/"] } } }]"#,
    );
    let config_text: String = config.to_string_lossy().into_owned();
    // Measured with coreutils `sha256sum` for the bytes `image bytes`.
    let url: &str = "https://lfs.example/de7030234493a8bea844dbe1d8676e68a2c1a4b014c721f0425a22b6df66faec/pkg/asset/shot.png";
    let arguments: [&str; 6] = [
        "--config",
        config_text.as_str(),
        "--stdin",
        "--stdin-filename",
        "pkg/README.md",
        "--fix-content",
    ];
    let fixed: Run = run(
        &repository.path,
        &arguments,
        b"# Title.\n\n![shot](asset/shot.png)\n\n![plain](asset/plain.svg)\n",
    );
    assert_eq!((fixed.status, fixed.stderr.as_str()), (0, ""));
    // Only the selected rule ran: the heading punctuation is untouched.
    assert_eq!(
        fixed.stdout,
        format!("# Title.\n\n![shot]({url})\n\n![plain](asset/plain.svg)\n")
    );
    let stale: String = format!(
        "![shot](https://lfs.example/{}/pkg/asset/gone.png)\n",
        "b".repeat(64)
    );
    let reported: Run = run(&repository.path, &arguments, stale.as_bytes());
    assert_eq!(reported.status, 1);
    assert_eq!(reported.stdout, stale);
    assert_eq!(
        located(reported.stderr.as_str()),
        ["pkg/README.md markdown/lfs-image-url 1 1"]
    );
    let excluded: Run = run(
        &repository.path,
        &[
            "--config",
            config_text.as_str(),
            "--stdin",
            "--stdin-filename",
            "package/ssg/a.md",
            "--fix-content",
        ],
        b"![shot](../../pkg/asset/shot.png)\n",
    );
    assert_eq!(
        (
            excluded.status,
            excluded.stdout.as_str(),
            excluded.stderr.as_str()
        ),
        (0, "![shot](../../pkg/asset/shot.png)\n", "")
    );
    // On-disk linting of the same repository reports the host file and never contacts the server.
    repository.write("pkg/README.md", "![shot](asset/shot.png)\n");
    let files: Run = run(
        &repository.path,
        &["--config", config_text.as_str(), "--fix-content"],
        b"",
    );
    assert_eq!(
        (files.status, files.stdout.as_str(), files.stderr.as_str()),
        (0, "", "")
    );
    assert_eq!(
        repository.read("pkg/README.md"),
        format!("![shot]({url})\n")
    );
}

/// The non-linting modes work on the real binary and never modify source files.
#[test]
fn rules_init_print_config_and_debug_modes() {
    let fixture: Fixture = Fixture::new();
    fixture.write("a.md", "# Title.\n");
    let rules: Run = run(&fixture.path, &["--rules"], b"");
    assert_eq!((rules.status, rules.stderr.as_str()), (0, ""));
    let listed: Vec<serde_json::Value> = records(rules.stdout.as_str());
    assert_listing_matches_registry(listed.as_slice());
    let init: Run = run(&fixture.path, &["--init"], b"");
    assert_eq!((init.status, init.stderr.as_str()), (0, ""));
    assert_eq!(
        init.stdout,
        format!("{}\n", fixture.path.join(CONFIG).display())
    );
    let starter: String = fixture.read(CONFIG);
    let again: Run = run(&fixture.path, &["--init"], b"");
    assert_eq!((again.status, again.stdout.as_str()), (2, ""));
    assert!(
        again
            .stderr
            .starts_with(format!("{PROGRAM}: Cannot create ").as_str()),
        "{}",
        again.stderr
    );
    assert_eq!(fixture.read(CONFIG), starter);
    // The starter configuration is immediately usable.
    let linted: Run = run(&fixture.path, &[], b"");
    assert_eq!(
        located(linted.stdout.as_str()),
        ["a.md markdown/no-trailing-punctuation 1 1"]
    );
    let printed: Run = run(&fixture.path, &["--print-config", "a.md"], b"");
    assert_eq!((printed.status, printed.stderr.as_str()), (0, ""));
    assert!(
        printed.stdout.contains("\"markdown/semantic-line-breaks\""),
        "{}",
        printed.stdout
    );
    let debug: Run = run(&fixture.path, &["--debug", "--concurrency", "1"], b"");
    assert_eq!(debug.stdout, linted.stdout);
    assert_eq!(debug.status, 1);
    assert!(!debug.stderr.is_empty());
    for line in debug.stderr.lines() {
        assert!(
            line.starts_with(format!("{PROGRAM}: debug: ").as_str()),
            "{line}"
        );
    }
    assert_eq!(fixture.read("a.md"), "# Title.\n");
}

/// `--debug` streams semantic-workspace progress to standard error while a workspace loads,
///  and a run without
/// it prints nothing there.
///  An unparsable `Cargo.toml` makes the load fail after its first progress message,
/// so both runs end quickly with the same single processing finding and status 2.
#[test]
fn debug_streams_workspace_progress_and_plain_runs_stay_silent() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        CONFIG,
        r#"[{ "files": ["**/*.rs"], "rules": { "rust/require-explicit-types": { "severity": "error" } } }]"#,
    );
    fixture.write("Cargo.toml", "[package\n");
    fixture.write("src/lib.rs", "//! Library.\n");
    let debug: Run = run(&fixture.path, &["--debug", "src/lib.rs"], b"");
    assert_eq!(debug.status, 2, "{}", debug.stderr);
    assert!(
        debug
            .stderr
            .contains(format!("{PROGRAM}: debug: discovering sysroot\n").as_str()),
        "{}",
        debug.stderr
    );
    let plain: Run = run(&fixture.path, &["src/lib.rs"], b"");
    assert_eq!((plain.status, plain.stderr.as_str()), (2, ""));
    // The messages quote Cargo's command line, which names a fresh temporary lockfile per run,
    // so the two runs are compared by finding location and code rather than byte for byte.
    for output in [&debug, &plain] {
        assert_eq!(
            located(output.stdout.as_str()),
            [format!("src/lib.rs {PROCESSING_FAILURE} 1 1")]
        );
    }
}

/// A write failure other than a closed pipe exits 2 whichever stream failed,
///  instead of the findings' status,
///  and names the refused stream and the operating system's error on the stream that still works.
/// `/dev/full` refuses every write with "no space left on device".
#[test]
fn a_refused_stream_write_exits_two_and_names_the_stream() {
    let fixture: Fixture = Fixture::new();
    fixture.write(CONFIG, RULES);
    fixture.write("a.md", "# Title.\n");
    let baseline: Run = run(&fixture.path, &[], b"");
    assert_eq!((baseline.status, baseline.stderr.as_str()), (1, ""));
    // Learn the refusal text from the device itself, so the expected line carries the kernel's own wording.
    let refusal: std::io::Error = std::fs::File::options()
        .write(true)
        .open("/dev/full")
        .expect("open /dev/full")
        .write_all(b"x")
        .expect_err("/dev/full refuses every write");
    // Standard output fails, so standard error carries the one explanation the run owes the reader.
    let full_stdout: std::fs::File = std::fs::File::options()
        .write(true)
        .open("/dev/full")
        .expect("open /dev/full");
    let stdout_failed: Output = Command::new(BINARY)
        .current_dir(&fixture.path)
        .stdin(Stdio::null())
        .stdout(Stdio::from(full_stdout))
        .stderr(Stdio::piped())
        .output()
        .expect("run the built executable");
    assert_eq!(stdout_failed.status.code(), Some(2));
    assert_eq!(
        String::from_utf8(stdout_failed.stderr).expect("UTF-8"),
        format!("{PROGRAM}: Cannot write standard output: {refusal}.\n")
    );
    // Standard error fails while `--debug` writes its notes there; the findings and the explanation
    // both reach standard output.
    let full_stderr: std::fs::File = std::fs::File::options()
        .write(true)
        .open("/dev/full")
        .expect("open /dev/full");
    let stderr_failed: Output = Command::new(BINARY)
        .arg("--debug")
        .current_dir(&fixture.path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::from(full_stderr))
        .output()
        .expect("run the built executable");
    assert_eq!(stderr_failed.status.code(), Some(2));
    assert_eq!(
        String::from_utf8(stderr_failed.stdout).expect("UTF-8"),
        format!(
            "{}{PROGRAM}: Cannot write standard error: {refusal}.\n",
            baseline.stdout
        )
    );
}

/// Run the executable as `run` does,
///  but with the main thread's stack limited to `kibibytes` by the shell.
/// `sh -c` receives the limit as `$0` and the executable with its arguments as `$@`,
///  so nothing is quoted by hand,
/// and `exec` replaces the shell,
///  so the status is the executable's own.
fn run_with_main_stack(
    cwd: &Path,
    kibibytes: &str,
    arguments: &[&str],
    input: &[u8],
) -> Run {
    let mut child: Child = Command::new("sh")
        .args(["-c", "ulimit -s \"$0\" && exec \"$@\"", kibibytes, BINARY])
        .args(arguments)
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start the built executable through sh");
    let mut stdin = child.stdin.take().expect("piped standard input");
    stdin.write_all(input).expect("write standard input");
    drop(stdin);
    let output: Output = child.wait_with_output().expect("wait for the executable");
    // A stack overflow aborts the process, which then has a signal and no exit code.
    let Some(status): Option<i32> = output.status.code() else {
        panic!(
            "the executable was killed by a signal: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    return Run {
        status,
        stdout: String::from_utf8(output.stdout).expect("standard output is UTF-8"),
        stderr: String::from_utf8(output.stderr).expect("standard error is UTF-8"),
    };
}

/// One file,
///  `--concurrency 1` and standard input are linted on a thread with the explicit lint stack,
///  not on the
/// main thread,
///  whose stack the platform sets (the MSVC linker reserves 1 MB by default).
///  With the main thread
/// limited to 1 MiB,
///  Rust nested 1,200 parentheses deep,
///  which needed between 3 and 4 MiB in the bounded container,
/// still parses in every mode.
///  Before the invocation thread existed,
///  all three runs aborted with a stack overflow.
#[test]
fn a_small_main_thread_stack_does_not_limit_nesting() {
    let fixture: Fixture = Fixture::new();
    fixture.write(CONFIG, RULES);
    // Documented items and one nested expression, so a completed check has no finding at all.
    let nested: String = format!(
        "//! Nested expression.\n\n/// Entry point.\nfn main() {{\n    let x: u32 = {}1{};\n}}\n",
        "(".repeat(1200),
        ")".repeat(1200)
    );
    fixture.write("a.rs", nested.as_str());
    fixture.write("b.rs", nested.as_str());
    let modes: [(&[&str], &[u8]); 3] = [
        (&["a.rs"], b""),
        (&["--concurrency", "1", "a.rs", "b.rs"], b""),
        (&["--stdin", "--stdin-filename", "a.rs"], nested.as_bytes()),
    ];
    for (arguments, input) in modes {
        let limited: Run = run_with_main_stack(&fixture.path, "1024", arguments, input);
        assert_eq!(
            (
                limited.status,
                limited.stdout.as_str(),
                limited.stderr.as_str()
            ),
            (0, "", ""),
            "{arguments:?}"
        );
    }
}

/// Output is the same at every concurrency limit,
///  and a reader that closes the pipe early causes no error output.
#[test]
fn concurrency_and_closed_pipes_do_not_change_results() {
    let fixture: Fixture = Fixture::new();
    fixture.write(CONFIG, RULES);
    for index in 0..300 {
        fixture.write(
            format!("d{}/f{index:03}.md", index % 7).as_str(),
            format!(
                "# Title {index}.\n\n# Second {index}\n\n### Third {index}\n\n```\ncode\n```\n"
            )
            .as_str(),
        );
    }
    let sequential: Run = run(&fixture.path, &["--concurrency", "1"], b"");
    assert_eq!((sequential.status, sequential.stderr.as_str()), (1, ""));
    assert_eq!(records(sequential.stdout.as_str()).len(), 1200);
    // More output than a pipe buffer holds, so an early close is observed by the writer.
    assert!(sequential.stdout.len() > 200_000);
    for limit in ["2", "4", "64"] {
        let parallel: Run = run(&fixture.path, &["--concurrency", limit], b"");
        assert_eq!(parallel.status, 1, "limit {limit}");
        assert_eq!(parallel.stderr, "", "limit {limit}");
        assert!(
            parallel.stdout == sequential.stdout,
            "limit {limit} changed the output"
        );
    }
    let default: Run = run(&fixture.path, &[], b"");
    assert!(default.stdout == sequential.stdout);
    // Start the executable, close the read end of its standard output at once, and collect what it says.
    let mut child: Child = Command::new(BINARY)
        .current_dir(&fixture.path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start the built executable");
    drop(child.stdout.take());
    let closed: Output = child.wait_with_output().expect("wait for the executable");
    assert_eq!(closed.status.code(), Some(1));
    assert_eq!(String::from_utf8(closed.stderr).expect("UTF-8"), "");
}

/// Inline directives through the real executable:
///  suppression,
///  the unused-directive options and their usage errors,
///  standard-input linting and the rule listing's suppressibility.
#[test]
fn directives_through_the_executable() {
    let fixture: Fixture = Fixture::new();
    fixture.write(CONFIG, RULES);
    fixture.write(
        "a.md",
        "# Title\n\n<!-- monochromatic-lint-disable-next-line markdown/single-h1 -- legacy title -->\n# Second\n",
    );
    fixture.write(
        "lib.rs",
        "//! Docs.\n// monochromatic-lint-disable-next-line rust/max-lines -- budget\n/// Item.\nfn item() {}\n",
    );
    // The Markdown directive suppresses its finding; the Rust directive names an unsuppressible rule.
    let plain: Run = run(&fixture.path, &[], b"");
    assert_eq!((plain.status, plain.stderr.as_str()), (1, ""));
    assert_eq!(
        located(plain.stdout.as_str()),
        [format!("lib.rs {NON_SUPPRESSIBLE} 2 41")]
    );
    // An unused directive is a warning on request, an error with the severity option.
    fixture.write("lib.rs", "//! Docs.\n");
    let warned: Run = run(&fixture.path, &["--report-unused-disable-directives"], b"");
    assert_eq!((warned.status, warned.stderr.as_str()), (0, ""));
    assert!(warned.stdout.is_empty(), "{}", warned.stdout);
    fixture.write("a.md", "# Title\n\n<!-- monochromatic-lint-disable-next-line markdown/single-h1 -- stale -->\nplain\n");
    let warning: Run = run(&fixture.path, &["--report-unused-disable-directives"], b"");
    assert_eq!(warning.status, 0);
    assert_eq!(records(warning.stdout.as_str())[0]["severity"], "warn");
    let error: Run = run(
        &fixture.path,
        &["--report-unused-disable-directives-severity", "error"],
        b"",
    );
    assert_eq!(error.status, 1);
    assert_eq!(records(error.stdout.as_str())[0]["code"], UNUSED);
    // Giving both spellings, or an unknown severity, is a usage error.
    for arguments in [
        vec![
            "--report-unused-disable-directives",
            "--report-unused-disable-directives-severity=warn",
        ],
        vec!["--report-unused-disable-directives-severity=loud"],
    ] {
        let refused: Run = run(&fixture.path, &arguments, b"");
        assert_eq!(
            (refused.status, refused.stdout.as_str()),
            (2, ""),
            "{arguments:?}"
        );
        assert!(
            refused
                .stderr
                .contains("--report-unused-disable-directives"),
            "{}",
            refused.stderr
        );
    }
    // Standard input is linted with the same directives.
    let piped: Run = run(
        &fixture.path,
        &[
            "--stdin",
            "--stdin-filename",
            "n.md",
            "--report-unused-disable-directives",
        ],
        b"# One\n\n<!-- monochromatic-lint-disable-next-line markdown/single-h1 -- r -->\nplain\n",
    );
    assert_eq!(
        located(piped.stdout.as_str()),
        [format!("n.md {UNUSED} 3 43")]
    );
    // The listing declares which rules refuse directives.
    let rules: Run = run(&fixture.path, &["--rules"], b"");
    let listed: Vec<serde_json::Value> = records(rules.stdout.as_str());
    assert_listing_matches_registry(listed.as_slice());
}

/// The built executable runs the two layout rules only where configuration selects them,
///  reads each file behind a `uses` link once,
///  names both folders and a proving reference,
///  and a package whose links match the code is clean.
#[cfg(unix)]
#[test]
fn layout_rules_run_in_the_built_executable_and_do_not_follow_links() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write("src/lib.rs", "mod top;\nmod middle;\n");
    // The marker is wrapped after its comma, as the Markdown rule demands.
    fixture.write("src/top/mod.rs", "//! Files in this folder,\n//! each using only files listed before it:\n//! - `user.rs`: uses the thing.\nmod user;\n");
    fixture.write("src/top/user.rs", "use crate::middle::Thing;\n");
    fixture.write("src/middle/mod.rs", "mod thing;\npub use thing::Thing;\n");
    fixture.write("src/middle/thing.rs", "pub struct Thing;\n");
    let configuration: &str = r#"[{ "files": ["**/*.rs"], "rules": {
      "rust/folder-uses-links": { "severity": "error" },
      "rust/file-order-in-folder": { "severity": "warn" },
      "rust/tests-in-sidecars": { "severity": "error" }
    } }]"#;
    fixture.write(CONFIG, configuration);
    // Off by default: the starter configuration names neither layout rule.
    fixture.write(
        "plain/Cargo.toml",
        "[package]\nname = \"plain\"\nversion = \"0.0.0\"\n",
    );
    fixture.write("plain/src/lib.rs", "mod a;\nmod b;\n");
    fixture.write("plain/src/a.rs", "use crate::b::X;\n");
    fixture.write("plain/src/b.rs", "use crate::a::X;\n");
    fixture.write("plain/monochromatic-lint.config.jsonc", r#"[{ "files": ["**/*.rs"], "rules": { "rust/tests-in-sidecars": { "severity": "error" } } }]"#);
    let off: Run = run(&fixture.path.join("plain"), &[], b"");
    assert_eq!(
        (off.status, off.stdout.as_str(), off.stderr.as_str()),
        (0, "", "")
    );
    // On: the use of `middle` by `top` has no link, and `src` has no file list.
    let missing: Run = run(&fixture.path, &["src"], b"");
    assert_eq!(missing.status, 1, "{}", missing.stderr);
    let found: Vec<String> = located(missing.stdout.as_str());
    assert!(
        found.contains(&String::from("src/top/user.rs rust/folder-uses-links 1 1")),
        "{found:?}"
    );
    assert!(
        missing
            .stdout
            .contains("src/top uses the folder src/middle (`crate::middle::Thing`)"),
        "{}",
        missing.stdout
    );
    // With the link in place, and a file list for the root folder, the layout findings are gone,
    // and a test function behind the link is reported once, not once per path to it.
    std::fs::create_dir_all(fixture.path.join("src/top/uses")).expect("uses folder");
    std::os::unix::fs::symlink("../../middle", fixture.path.join("src/top/uses/middle"))
        .expect("link");
    let list: &str = "//! Files in this folder, each using only files listed before it:\n//! - `thing.rs`: the thing.\n";
    fixture.write(
        "src/middle/mod.rs",
        format!("{list}mod thing;\npub use thing::Thing;\n#[test]\nfn stray() {{}}\n").as_str(),
    );
    fixture.write("src/lib.rs", "//! Files in this folder, each using only files listed before it:\nmod top;\nmod middle;\n");
    let linked: Run = run(&fixture.path, &["src"], b"");
    assert_eq!(
        located(linked.stdout.as_str()),
        ["src/middle/mod.rs rust/tests-in-sidecars 6 4"],
        "{}",
        linked.stderr
    );
}

/// A module file that two `mod` declarations reach is one module,
///  so one glob re-export in it is one finding with one key,
///  while two globs written on one line stay two findings.
#[test]
fn layout_a_module_file_reached_twice_reports_each_glob_once() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    // The library and the executable both declare the same three modules,
    // which is what a package whose two roots share its module tree looks like.
    fixture.write(
        "src/lib.rs",
        "//! Index.\npub mod a;\npub mod b;\npub mod shared;\n",
    );
    fixture.write(
        "src/main.rs",
        "//! Entry.\nfn main() {}\npub mod a;\npub mod b;\npub mod shared;\n",
    );
    fixture.write("src/a.rs", "pub struct X;\n");
    fixture.write("src/b.rs", "pub struct Y;\n");
    fixture.write("src/shared.rs", "pub use crate::a::*;\n");
    fixture.write(
        CONFIG,
        r#"[{ "files": ["src/**/*.rs"], "rules": { "rust/folder-uses-links": { "severity": "error" } } }]"#,
    );
    // One source line, reached through both declarations, is one finding.
    let found: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(found.status, 1, "{}", found.stderr);
    assert_eq!(
        located(found.stdout.as_str()),
        [String::from("src/shared.rs rust/folder-uses-links 1 1")],
        "{}",
        found.stdout
    );
    assert_eq!(
        found
            .stderr
            .lines()
            .filter(|line| return line.contains("allow-list key:"))
            .collect::<Vec<&str>>(),
        [
            "monochromatic-lint: debug: rust/folder-uses-links: allow-list key: glob src/shared.rs crate::a"
        ],
        "{}",
        found.stderr
    );
    // Two globs on that one line stay two findings, each with its own key, once each.
    fixture.write("src/shared.rs", "pub use crate::{a::*, b::*};\n");
    let two: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(two.status, 1, "{}", two.stderr);
    assert_eq!(
        two.stderr
            .lines()
            .filter(|line| return line.contains("allow-list key:"))
            .collect::<Vec<&str>>(),
        [
            "monochromatic-lint: debug: rust/folder-uses-links: allow-list key: glob src/shared.rs crate::a",
            "monochromatic-lint: debug: rust/folder-uses-links: allow-list key: glob src/shared.rs crate::b",
        ],
        "{}",
        two.stderr
    );
}

/// Two glob re-exports written on one line are two findings with two keys,
///  so a list line recorded for the first never tolerates the second,
///  and that line stays alive while its own glob exists.
#[test]
fn layout_two_globs_on_one_line_are_two_findings_in_the_built_executable() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(
        "src/lib.rs",
        "//! Index.\npub mod a;\npub mod b;\npub use crate::{a::*, b::*};\n",
    );
    fixture.write("src/a.rs", "pub struct X;\n");
    fixture.write("src/b.rs", "pub struct Y;\n");
    // The list records only the first of the two globs the line holds.
    fixture.write("folder.txt", "glob src/lib.rs crate::a\n");
    fixture.write(
        CONFIG,
        r#"[{ "files": ["src/**/*.rs"], "rules": { "rust/folder-uses-links": { "severity": "error", "allow": "folder.txt" } } }]"#,
    );
    let found: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(found.status, 1, "{}", found.stderr);
    // The second glob is the finding; the listed first one tolerates itself and its line stays alive.
    assert_eq!(
        located(found.stdout.as_str()),
        [String::from("src/lib.rs rust/folder-uses-links 4 1")],
        "{}",
        found.stdout
    );
    assert!(
        found
            .stdout
            .contains("`pub use crate::b::*` re-exports every item"),
        "{}",
        found.stdout
    );
    assert!(
        found
            .stderr
            .contains("rust/folder-uses-links: allow-list key: glob src/lib.rs crate::b"),
        "{}",
        found.stderr
    );
    assert!(
        !found.stderr.contains("tolerates nothing any more"),
        "{}",
        found.stderr
    );
}

/// The built executable checks group folders:
///  a group's `uses` summary is wanted when a child uses another group,
///  accepted once it links the group,
///  and a child that links the group itself is told so.
#[cfg(unix)]
#[test]
fn layout_group_folders_run_in_the_built_executable() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write("src/lib.rs", "mod g;\nmod h;\n");
    let list: &str = "//! Files in this folder, each using only files listed before it:\n//!\n";
    fixture.write("src/g/mod.rs", "mod a;\n");
    fixture.write(
        "src/g/a/mod.rs",
        format!("{list}//! - `thing.rs`: the thing.\nmod thing;\n").as_str(),
    );
    fixture.write("src/g/a/thing.rs", "pub struct X;\n");
    fixture.write("src/h/mod.rs", "mod c;\n");
    fixture.write(
        "src/h/c/mod.rs",
        format!("{list}//! - `user.rs`: uses the thing.\nmod user;\n").as_str(),
    );
    fixture.write("src/h/c/user.rs", "use crate::g::a::X;\n");
    fixture.write(
        CONFIG,
        r#"[{ "files": ["**/*.rs"], "rules": {
      "rust/folder-uses-links": { "severity": "error" },
      "rust/file-order-in-folder": { "severity": "error" }
    } }]"#,
    );
    std::fs::create_dir_all(fixture.path.join("src/h/c/uses")).expect("child uses folder");
    std::os::unix::fs::symlink("../../../g/a", fixture.path.join("src/h/c/uses/a"))
        .expect("child link");
    // The child's link is exact, but the group has no summary of the entry its child uses.
    let missing: Run = run(&fixture.path, &["src"], b"");
    assert_eq!(missing.status, 1, "{}", missing.stderr);
    assert_eq!(
        located(missing.stdout.as_str()),
        ["src/h/c/user.rs rust/folder-uses-links 1 1"],
        "{}",
        missing.stdout
    );
    assert!(
        missing.stdout.contains("src/h/uses has no link to src/g."),
        "{}",
        missing.stdout
    );
    // With the summary in place nothing is left.
    std::fs::create_dir_all(fixture.path.join("src/h/uses")).expect("group uses folder");
    std::os::unix::fs::symlink("../../g", fixture.path.join("src/h/uses/g")).expect("group link");
    let clean: Run = run(&fixture.path, &["src"], b"");
    assert_eq!(
        (clean.status, clean.stdout.as_str()),
        (0, ""),
        "{}",
        clean.stderr
    );
}

/// A use by a group's own file reaches no entry edge,
///  because the shelf rule reports that file and a group's summary is what its child folders use,
///  so the loop it closes is reported by the group-own check,
///  and no `uses` line records that use.
#[test]
fn layout_a_groups_own_file_use_inside_a_tolerated_loop_is_reported() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write("src/lib.rs", "//! Index.\npub mod a;\npub mod grp;\n");
    fixture.write("src/a/mod.rs", "//! A.\nuse crate::grp::g2::X;\n");
    fixture.write(
        "src/grp/mod.rs",
        "//! Group.\npub mod own;\npub mod g1;\npub mod g2;\n",
    );
    // The group's own file. The shelf rule reports it; its use reaches no entry loop.
    fixture.write("src/grp/own.rs", "//! Own.\nuse crate::a::X;\n");
    fixture.write("src/grp/g1/mod.rs", "//! G1.\nuse crate::a::X;\n");
    fixture.write("src/grp/g2/mod.rs", "//! G2.\n");
    // Every finding of this tree except the use in the group's own file,
    // and both uses that hold the entry loop between `a` and `grp`.
    fixture.write(
        "folder.txt",
        "missing-link src/a -> src/grp/g2\n\
         missing-link src/grp/g1 -> src/a\n\
         group-file src/grp own.rs\n\
         summary-missing src/grp src/a\n\
         uses src/a -> src/grp/g2\n\
         uses src/grp/g1 -> src/a\n",
    );
    fixture.write(
        CONFIG,
        r#"[{ "files": ["src/**/*.rs"], "rules": { "rust/folder-uses-links": { "severity": "error", "allow": "folder.txt" } } }]"#,
    );
    let clean: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(clean.status, 1, "{}", clean.stderr);
    // The use closes a loop the entry graph cannot see, and the finding carries no key.
    for part in [
        "These entries use each other in a loop",
        "src/grp uses src/a",
    ] {
        assert!(
            clean.stdout.contains(part),
            "{part} missing in {}",
            clean.stdout
        );
    }
    // No `uses` line records that use: a line naming it is the dead line below.
    fixture.write(
        "folder.txt",
        "missing-link src/a -> src/grp/g2\n\
         missing-link src/grp/g1 -> src/a\n\
         group-file src/grp own.rs\n\
         summary-missing src/grp src/a\n\
         uses src/a -> src/grp/g2\n\
         uses src/grp/g1 -> src/a\n\
         uses src/grp -> src/a\n",
    );
    let dead: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(dead.status, 1, "{}", dead.stderr);
    assert!(
        dead.stdout
            .contains("uses src/grp -> src/a` in folder.txt tolerates nothing any more"),
        "{}",
        dead.stdout
    );
    assert!(
        dead.stdout
            .contains("These entries use each other in a loop"),
        "{}",
        dead.stdout
    );
}

/// A folder whose index cannot be read reports the index itself
///  and one missing entry per file the folder holds,
///  so the `unreadable-index` line tolerates only its own finding
///  and a file added to that folder is still reported.
#[test]
fn layout_an_unreadable_index_reports_every_file_it_cannot_list() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    // The folder's files are declared from outside it, so the module tree never reads the index
    // and the run reaches the file rule's own read of it.
    fixture.write(
        "src/lib.rs",
        "#[path = \"a/b.rs\"] mod b;\n#[path = \"a/c.rs\"] mod c;\n",
    );
    fixture.write("src/a/b.rs", "\n");
    fixture.write("src/a/c.rs", "\n");
    // An index that is not UTF-8 is unreadable to every user, not only to one without a permission bit.
    std::fs::write(fixture.path.join("src/a/mod.rs"), [0x2f, 0x2f, 0xff, b'\n'])
        .expect("fixture index");
    fixture.write(
        CONFIG,
        r#"[{ "files": ["src/**/*.rs"], "rules": { "rust/file-order-in-folder": { "severity": "error" } } }]"#,
    );
    // Name the readable files, so the unreadable index is read by the layout rules alone.
    let found: Run = run(
        &fixture.path,
        &["--debug", "src/lib.rs", "src/a/b.rs", "src/a/c.rs"],
        b"",
    );
    assert_eq!(found.status, 1, "{}", found.stderr);
    assert_eq!(
        located(found.stdout.as_str()),
        [
            String::from("src/a/mod.rs rust/file-order-in-folder 1 1"),
            String::from("src/a/b.rs rust/file-order-in-folder 1 1"),
            String::from("src/a/c.rs rust/file-order-in-folder 1 1"),
        ],
        "{}",
        found.stdout
    );
    // The index line holds only its own finding; each file it could not list has its own key.
    for key in [
        "rust/file-order-in-folder: allow-list key: unreadable-index src/a/mod.rs",
        "rust/file-order-in-folder: allow-list key: missing-entry src/a/mod.rs b.rs",
        "rust/file-order-in-folder: allow-list key: missing-entry src/a/mod.rs c.rs",
    ] {
        assert!(
            found.stderr.contains(key),
            "{key} missing in {}",
            found.stderr
        );
    }
}

/// A module file whose name holds a space is reported by a finding whose key names the file,
///  so the line that tolerates it never passes to a file renamed later,
///  and the file still owes its folder a list entry.
#[test]
fn layout_a_file_name_holding_a_space_is_refused_in_the_built_executable() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(
        "src/lib.rs",
        "//! Files in this folder,\n//! each using only files listed before it:\n#[path = \"a b.rs\"] mod ab;\n",
    );
    fixture.write("src/a b.rs", "pub struct X;\n");
    fixture.write(
        CONFIG,
        r#"[{ "files": ["src/**/*.rs"], "rules": { "rust/file-order-in-folder": { "severity": "error" } } }]"#,
    );
    let found: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(found.status, 1, "{}", found.stderr);
    assert_eq!(
        located(found.stdout.as_str()),
        [
            String::from("src/a b.rs rust/file-order-in-folder 1 1"),
            String::from("src/lib.rs rust/file-order-in-folder 1 1"),
        ],
        "{}",
        found.stdout
    );
    assert!(
        found
            .stdout
            .contains("a b.rs in the folder src holds a space"),
        "{}",
        found.stdout
    );
    // The message names the line's real word count, so the reader writes the shape the refusal also prints.
    assert!(
        found
            .stdout
            .contains("a use between two files is written as five words"),
        "{}",
        found.stdout
    );
    // Both lines name the file, so neither passes to another name.
    let key_lines: Vec<&str> = found
        .stderr
        .lines()
        .filter(|line| return line.contains("allow-list key:"))
        .collect::<Vec<&str>>();
    assert_eq!(
        key_lines,
        [
            "monochromatic-lint: debug: rust/file-order-in-folder: allow-list key: space-in-file-name src a b.rs",
            "monochromatic-lint: debug: rust/file-order-in-folder: allow-list key: missing-entry src/lib.rs a b.rs",
        ],
        "{}",
        found.stderr
    );
}

/// A module file whose name ends in a space, a tab or a no-break space is refused before either rule runs,
///  because the list reader trims each line,
///  so no line could hold a key ending in that name.
#[test]
fn layout_a_module_file_name_that_ends_in_whitespace_is_refused() {
    for (name, escaped) in [
        ("x.rs ", "x.rs\\u{20}"),
        ("x.rs\t", "x.rs\\u{9}"),
        ("x.rs\u{a0}", "x.rs\\u{a0}"),
    ] {
        let fixture: Fixture = Fixture::new();
        fixture.write(
            "Cargo.toml",
            "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
        );
        fixture.write(
            "src/lib.rs",
            format!("#[path = \"{name}\"] mod x;\n").as_str(),
        );
        fixture.write(format!("src/{name}").as_str(), "pub struct X;\n");
        fixture.write(
            CONFIG,
            r#"[{ "files": ["src/**/*.rs"], "rules": { "rust/folder-uses-links": { "severity": "error" }, "rust/file-order-in-folder": { "severity": "error" } } }]"#,
        );
        let refused: Run = run(&fixture.path, &["src"], b"");
        assert_eq!(refused.status, 2, "{escaped}: {}", refused.stderr);
        let reported: Vec<serde_json::Value> = records(refused.stdout.as_str());
        assert_eq!(reported.len(), 1, "{escaped}: {}", refused.stdout);
        // The message spells the character a reader cannot see, so it names the file exactly.
        let message: &str = reported[0]["message"].as_str().expect("message");
        for part in [
            format!("The layout rules stop at src/{escaped},").as_str(),
            "trims away",
        ] {
            assert!(message.contains(part), "{escaped}: {message}");
        }
        assert_eq!(reported[0]["code"], PROCESSING_FAILURE, "{message}");
    }
}

/// A `mod.rs` that no `mod` line declares is not its folder's index:
///  the layout rules read the module tree, so a leftover file moves no key,
///  hides no code check of the folder's real index, and makes no group.
#[test]
fn layout_an_undeclared_index_file_changes_nothing() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    // The index holds code and lists no file, and no `mod` line reaches the stray file below.
    fixture.write("src/lib.rs", "pub fn code_in_index() {}\nmod a;\n");
    fixture.write("src/a.rs", "\n");
    fixture.write(
        CONFIG,
        r#"[{ "files": ["src/**/*.rs"], "rules": {
      "rust/folder-uses-links": { "severity": "error" },
      "rust/file-order-in-folder": { "severity": "error" }
    } }]"#,
    );
    let without: Run = run(&fixture.path, &["--debug", "src/lib.rs", "src/a.rs"], b"");
    assert_eq!(without.status, 1, "{}", without.stderr);
    fixture.write("src/mod.rs", "//! Stray.\n");
    // The stray file is not part of the package, so every finding stays keyed to the real index.
    let stray: Run = run(
        &fixture.path,
        &["--debug", "src/lib.rs", "src/a.rs", "src/mod.rs"],
        b"",
    );
    assert_eq!(stray.status, 1, "{}", stray.stderr);
    assert_eq!(
        located(stray.stdout.as_str()),
        located(without.stdout.as_str()),
        "{}\n{}",
        without.stdout,
        stray.stdout
    );
    for key in [
        "rust/file-order-in-folder: allow-list key: index-item src/lib.rs function-code_in_index",
        "rust/file-order-in-folder: allow-list key: no-file-list src/lib.rs",
        "rust/file-order-in-folder: allow-list key: missing-entry src/lib.rs a.rs",
    ] {
        assert!(
            stray.stderr.contains(key),
            "{key} missing in {}",
            stray.stderr
        );
    }
    // The same file beside two child folders makes no group of the folder it sits in.
    let group: Fixture = Fixture::new();
    group.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    group.write("src/lib.rs", "//! Index.\nmod a;\nmod b;\n");
    group.write("src/a/mod.rs", "//! A.\nmod x;\npub use x::X;\n");
    group.write("src/a/x.rs", "pub struct X;\n");
    group.write("src/b/mod.rs", "//! B.\nmod y;\n");
    group.write("src/b/y.rs", "use crate::a::X;\n");
    group.write(
        CONFIG,
        r#"[{ "files": ["src/**/*.rs"], "rules": {
      "rust/folder-uses-links": { "severity": "error" }
    } }]"#,
    );
    std::fs::create_dir_all(group.path.join("src/b/uses")).expect("uses folder");
    std::os::unix::fs::symlink("../../a", group.path.join("src/b/uses/a")).expect("link");
    let shelf: Run = run(&group.path, &["src"], b"");
    assert_eq!(
        (shelf.status, shelf.stdout.as_str()),
        (0, ""),
        "{}",
        shelf.stderr
    );
    group.write("src/mod.rs", "//! Stray.\n");
    let stray_group: Run = run(&group.path, &["src"], b"");
    assert_eq!(
        (stray_group.status, stray_group.stdout.as_str()),
        (0, ""),
        "{}",
        stray_group.stderr
    );
}

/// A module file in the package folder itself is keyed by `.`,
///  the one spelling of the package folder that a list line can hold,
///  so no key ends in the space the list reader trims away.
#[test]
fn layout_a_module_file_in_the_package_folder_has_a_matchable_key() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    // The package folder holds `x.rs` and no index of its own, so its keys are about the folder itself.
    fixture.write("src/lib.rs", "#[path = \"../x.rs\"] pub mod x;\n");
    fixture.write("x.rs", "pub struct X;\n");
    fixture.write(
        CONFIG,
        r#"[{ "files": ["src/**/*.rs", "x.rs"], "rules": { "rust/file-order-in-folder": { "severity": "error" } } }]"#,
    );
    let found: Run = run(&fixture.path, &["--debug", "src/lib.rs", "x.rs"], b"");
    assert_eq!(found.status, 1, "{}", found.stderr);
    for key in [
        "rust/file-order-in-folder: allow-list key: no-top-doc .",
        "rust/file-order-in-folder: allow-list key: missing-entry . x.rs",
    ] {
        assert!(
            found.stderr.contains(key),
            "{key} missing in {}",
            found.stderr
        );
    }
    // Both lines are copied from the keys, and the list then holds every finding.
    fixture.write("folder.txt", "no-top-doc .\nmissing-entry . x.rs\n");
    fixture.write(
        CONFIG,
        r#"[{ "files": ["src/**/*.rs", "x.rs"], "rules": { "rust/file-order-in-folder": { "severity": "error", "allow": "folder.txt" } } }]"#,
    );
    let listed: Run = run(&fixture.path, &["--debug", "src/lib.rs", "x.rs"], b"");
    assert_eq!(
        (listed.status, listed.stdout.as_str()),
        (0, ""),
        "{}",
        listed.stderr
    );
    // A loop through the package folder itself is tolerated by lines that name it `.`.
    let usage: Fixture = Fixture::new();
    usage.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n\n[lib]\npath = \"lib.rs\"\n",
    );
    usage.write("lib.rs", "//! Index.\nmod a;\nmod sub;\n");
    usage.write("a.rs", "pub struct X;\nuse crate::sub::Y;\n");
    usage.write("sub/mod.rs", "//! Sub.\npub struct Y;\nuse crate::a::X;\n");
    usage.write(
        CONFIG,
        r#"[{ "files": ["*.rs", "sub/**/*.rs"], "rules": { "rust/folder-uses-links": { "severity": "error", "allow": "folder.txt" } } }]"#,
    );
    // Without the list the loop is reported; both of its uses are then listed.
    usage.write("folder.txt", "");
    let looped: Run = run(&usage.path, &["--debug", "lib.rs", "a.rs"], b"");
    assert_eq!(looped.status, 1, "{}", looped.stderr);
    assert!(
        looped
            .stderr
            .contains("allow-list key: missing-link . -> sub"),
        "{}",
        looped.stderr
    );
    usage.write(
        "folder.txt",
        "uses . -> sub\nuses sub -> .\nmissing-link . -> sub\nmissing-link sub -> .\n",
    );
    let linked: Run = run(&usage.path, &["--debug", "lib.rs", "a.rs"], b"");
    assert_eq!(
        (linked.status, linked.stdout.as_str()),
        (0, ""),
        "{}",
        linked.stderr
    );
}

/// A module file whose name holds a space stays in the list, loop and order checks:
///  a loop through it is reported,
///  and its key names the file, so a list line never tolerates a rename.
#[test]
fn layout_a_file_name_holding_a_space_stays_in_every_check() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write("src/lib.rs", "//! Index.\nmod m;\nmod n;\n");
    // The spaced file is declared by `#[path]`, which is the one way a module file can carry a space.
    fixture.write(
        "src/m/mod.rs",
        "//! Files in this folder,\n//! each using only files listed before it:\n//! - `p.rs`: the p.\n//! - `q.rs`: the q.\n//! - `p q.rs`: the spaced one.\n#[path = \"p q.rs\"] mod pq;\nmod p;\nmod q;\n",
    );
    fixture.write("src/m/p.rs", "pub struct X;\nuse super::pq::Y;\n");
    fixture.write("src/m/q.rs", "pub struct Q;\n");
    fixture.write("src/m/p q.rs", "pub struct Y;\nuse super::p::X;\n");
    // A folder whose one spaced file no other file uses, so only its own key is in play.
    fixture.write(
        "src/n/mod.rs",
        "//! Files in this folder,\n//! each using only files listed before it:\n//! - `p.rs`: the p.\n//! - `p q.rs`: the spaced one.\n#[path = \"p q.rs\"] mod pq;\nmod p;\n",
    );
    fixture.write("src/n/p.rs", "pub struct X;\n");
    fixture.write("src/n/p q.rs", "pub struct Y;\n");
    fixture.write(
        CONFIG,
        r#"[{ "files": ["src/**/*.rs"], "rules": { "rust/file-order-in-folder": { "severity": "error" } } }]"#,
    );
    // The loop through the spaced file is reported, and its own line names the file.
    let looped: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(looped.status, 1, "{}", looped.stderr);
    assert!(
        looped.stdout.contains("use each other in a loop"),
        "{}",
        looped.stdout
    );
    for part in ["p q.rs uses p.rs", "p.rs uses p q.rs"] {
        assert!(
            looped.stdout.contains(part),
            "{part} missing in {}",
            looped.stdout
        );
    }
    assert!(
        looped
            .stderr
            .contains("allow-list key: space-in-file-name src/m p q.rs"),
        "{}",
        looped.stderr
    );
    // A line for the spaced file tolerates its own finding and never the loop.
    std::fs::write(
        fixture.path.join("folder.txt"),
        "space-in-file-name src/m p q.rs\n",
    )
    .expect("fixture list");
    fixture.write(
        CONFIG,
        r#"[{ "files": ["src/**/*.rs"], "rules": { "rust/file-order-in-folder": { "severity": "error", "allow": "folder.txt" } } }]"#,
    );
    let tolerated: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(tolerated.status, 1, "{}", tolerated.stderr);
    assert!(
        tolerated.stdout.contains("use each other in a loop"),
        "{}",
        tolerated.stdout
    );
    // The other folder's line holds, and a rename makes it dead instead of letting it pass.
    std::fs::write(
        fixture.path.join("folder.txt"),
        "space-in-file-name src/n p q.rs\n",
    )
    .expect("fixture list");
    let clean: Run = run(&fixture.path, &["--debug", "src"], b"");
    // The line holds: no finding of that folder is left, and the other folder's loop is still reported.
    assert_eq!(clean.status, 1, "{}", clean.stderr);
    assert!(
        !clean.stdout.contains("\"filename\":\"src/n"),
        "{}",
        clean.stdout
    );
    std::fs::remove_file(fixture.path.join("src/n/p q.rs")).expect("remove the spaced file");
    fixture.write("src/n/x y.rs", "pub struct Y;\n");
    fixture.write(
        "src/n/mod.rs",
        "//! Files in this folder,\n//! each using only files listed before it:\n//! - `p.rs`: the p.\n//! - `x y.rs`: the spaced one.\n#[path = \"x y.rs\"] mod pq;\nmod p;\n",
    );
    let renamed: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(renamed.status, 1, "{}", renamed.stderr);
    assert!(
        renamed
            .stdout
            .contains("space-in-file-name src/n p q.rs` in folder.txt tolerates nothing any more"),
        "{}",
        renamed.stdout
    );
}

/// A use written in a group's own file that closes a mutual dependency between its group and another entry is reported,
///  with no key,
///  because a group's own code makes no edge of the entry graph
///  and the allow list promises that new code cannot add a loop.
#[test]
fn layout_a_groups_own_file_use_that_closes_a_loop_is_reported() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write("src/lib.rs", "//! Index.\npub mod a;\npub mod grp;\n");
    // `a` uses the group's second folder, so the group already uses `a` once the own file joins in.
    fixture.write("src/a/mod.rs", "//! A.\nmod one;\n");
    fixture.write("src/a/one.rs", "pub struct Z;\nuse crate::grp::g2::Y;\n");
    fixture.write(
        "src/grp/mod.rs",
        "//! Group.\npub mod g1;\npub mod g2;\npub mod own;\n",
    );
    fixture.write("src/grp/own.rs", "//! Own.\nuse crate::a::one::Z;\n");
    fixture.write("src/grp/g1/mod.rs", "//! G1.\n");
    fixture.write("src/grp/g2/mod.rs", "//! G2.\npub struct Y;\n");
    // Every other finding of this tree is on the list, so only the loop is left to report.
    fixture.write(
        "folder.txt",
        "group-file src/grp own.rs\nmissing-link src/a -> src/grp/g2\n",
    );
    fixture.write(
        CONFIG,
        r#"[{ "files": ["src/**/*.rs"], "rules": { "rust/folder-uses-links": { "severity": "error", "allow": "folder.txt" } } }]"#,
    );
    let looped: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(looped.status, 1, "{}", looped.stderr);
    for part in [
        "use each other in a loop",
        "src/grp uses src/a",
        "src/a uses src/grp",
    ] {
        assert!(
            looped.stdout.contains(part),
            "{part} missing in {}",
            looped.stdout
        );
    }
    // The use is what closes the loop: without it the same list holds every finding.
    fixture.write("src/grp/own.rs", "//! Own.\n");
    let quiet: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(
        (quiet.status, quiet.stdout.as_str()),
        (0, ""),
        "{}",
        quiet.stderr
    );
}

/// A mutual dependency between two groups that two own-file uses close is reported,
///  as one that a single own-file use closes already is:
///  the walk back from the entry a group's own file uses steps over another group's own-file use too.
#[test]
fn layout_a_mutual_dependency_between_groups_closed_by_two_own_file_uses_is_reported() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write("src/lib.rs", "//! Index.\npub mod g1;\npub mod g2;\n");
    fixture.write("src/g1/mod.rs", "//! G1.\npub mod own;\npub mod c1;\n");
    // The first group's own file uses a folder of the second group.
    fixture.write("src/g1/own.rs", "//! Own.\nuse crate::g2::c2::X;\n");
    fixture.write("src/g1/c1/mod.rs", "//! C1.\npub struct Y;\n");
    fixture.write("src/g2/mod.rs", "//! G2.\npub mod own;\npub mod c2;\n");
    // The second group's own file uses a folder of the first, which closes the loop.
    fixture.write("src/g2/own.rs", "//! Own.\nuse crate::g1::c1::Y;\n");
    fixture.write("src/g2/c2/mod.rs", "//! C2.\npub struct X;\n");
    fixture.write(
        CONFIG,
        r#"[{ "files": ["src/**/*.rs"], "rules": { "rust/folder-uses-links": { "severity": "error" } } }]"#,
    );
    let looped: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(looped.status, 1, "{}", looped.stderr);
    // Each own-file use is reported, naming itself and the whole path back to its group.
    for part in [
        "These entries use each other in a loop",
        "src/g1 uses src/g2/c2 (`crate::g2::c2::X`); src/g2 uses src/g1 (`crate::g1::c1::Y` at src/g2/own.rs:2)",
        "src/g2 uses src/g1/c1 (`crate::g1::c1::Y`); src/g1 uses src/g2 (`crate::g2::c2::X` at src/g1/own.rs:2)",
    ] {
        assert!(
            looped.stdout.contains(part),
            "{part} missing in {}",
            looped.stdout
        );
    }
}

/// The finding about a use written in a group's own file names that use in its key,
///  so a tree that already holds one can record it on a list,
///  and a use added later still cannot be recorded anywhere.
#[test]
fn layout_the_own_file_loop_finding_is_keyed_by_its_use() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write("src/lib.rs", "//! Index.\npub mod a;\npub mod grp;\n");
    fixture.write("src/a/mod.rs", "//! A.\nuse crate::grp::g2::X;\n");
    fixture.write("src/grp/mod.rs", "//! Group.\npub mod own;\npub mod g2;\n");
    fixture.write("src/grp/own.rs", "//! Own.\nuse crate::a::X;\n");
    fixture.write("src/grp/g2/mod.rs", "//! G2.\npub struct X;\n");
    fixture.write(
        CONFIG,
        r#"[{ "files": ["src/**/*.rs"], "rules": { "rust/folder-uses-links": { "severity": "error", "allow": "folder.txt" } } }]"#,
    );
    // Every finding of the tree except the loop's own-file use, which no `uses` line can record,
    // and the use inside the loop, which its own `uses` line records.
    fixture.write(
        "folder.txt",
        "missing-link src/a -> src/grp/g2\ngroup-file src/grp own.rs\nuses src/a -> src/grp/g2\n",
    );
    let keyless: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(keyless.status, 1, "{}", keyless.stderr);
    assert!(
        keyless
            .stdout
            .contains("These entries use each other in a loop"),
        "{}",
        keyless.stdout
    );
    // The rule prints the key of every finding it keeps on the debug stream.
    assert!(
        keyless
            .stderr
            .contains("allow-list key: own-file-loop src/grp -> src/a"),
        "{}",
        keyless.stderr
    );
    // The line names that use, and the loop's other use has its own line, so the list holds every finding of this tree.
    fixture.write(
        "folder.txt",
        "missing-link src/a -> src/grp/g2\ngroup-file src/grp own.rs\nown-file-loop src/grp -> src/a\nuses src/a -> src/grp/g2\n",
    );
    let listed: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(
        (listed.status, listed.stdout.as_str()),
        (0, ""),
        "{}",
        listed.stderr
    );
    // The use is what the line records: without it the line tolerates nothing.
    fixture.write("src/grp/own.rs", "//! Own.\n");
    let dead: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(dead.status, 1, "{}", dead.stderr);
    assert!(
        dead.stdout
            .contains("own-file-loop src/grp -> src/a` in folder.txt tolerates nothing any more"),
        "{}",
        dead.stdout
    );
}

/// `--config` reads an allow path from the working directory,
///  which is also the base of that configuration's own file patterns,
///  so the list is found where the run was started and not beside the configuration file.
#[test]
fn layout_a_config_allow_path_is_read_from_the_working_directory() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "pkg/Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write("pkg/src/lib.rs", "pub fn code_in_index() {}\nmod a;\n");
    fixture.write("pkg/src/a.rs", "\n");
    // The list at the working directory holds every finding this run makes.
    fixture.write(
        "folder.txt",
        "index-item src/lib.rs function-code_in_index\nno-file-list src/lib.rs\nmissing-entry src/lib.rs a.rs\n",
    );
    // A list beside the configuration holds a line that tolerates nothing,
    // so reading that one would fail the run.
    fixture.write("nested/folder.txt", "no-top-doc src\n");
    fixture.write(
        "nested/config.jsonc",
        r#"[{ "files": ["pkg/src/**/*.rs"], "rules": { "rust/file-order-in-folder": { "severity": "error", "allow": "folder.txt" } } }]"#,
    );
    let found: Run = run(
        &fixture.path,
        &["--config", "nested/config.jsonc", "--debug", "pkg/src"],
        b"",
    );
    assert_eq!(
        (found.status, found.stdout.as_str()),
        (0, ""),
        "{}",
        found.stderr
    );
}

/// A `file-uses` file word is read as written, backslash and all,
///  because a file name is not a path and a key keeps its backslash too.
#[cfg(unix)]
#[test]
fn layout_a_file_uses_line_names_a_file_with_a_backslash() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "pkg/Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write("pkg/src/lib.rs", "//! Index.\nmod a;\n");
    // The attribute's source spells the backslash twice, and the file name holds it once.
    fixture.write(
        "pkg/src/a/mod.rs",
        "//! Files in this folder,\n//! each using only files listed before it:\n//! - `p.rs`: the p.\n//! - `x\\y.rs`: the backslash one.\n#[path = \"x\\\\y.rs\"] mod xy;\nmod p;\n",
    );
    fixture.write("pkg/src/a/p.rs", "pub struct X;\nuse super::xy::Y;\n");
    fixture.write("pkg/src/a/x\\y.rs", "pub struct Y;\nuse super::p::X;\n");
    fixture.write(
        "pkg/monochromatic-lint.config.jsonc",
        r#"[{ "files": ["src/**/*.rs"], "rules": { "rust/file-order-in-folder": { "severity": "error", "allow": "folder.txt" } } }]"#,
    );
    // Both uses of the loop are listed, and each line names the backslash file as written.
    fixture.write(
        "pkg/folder.txt",
        "file-uses src/a p.rs -> x\\y.rs\nfile-uses src/a x\\y.rs -> p.rs\n",
    );
    let found: Run = run(&fixture.path.join("pkg"), &["--debug", "src"], b"");
    assert_eq!(
        (found.status, found.stdout.as_str()),
        (0, ""),
        "{}",
        found.stderr
    );
}

/// A module file a `#[path]` names outside the package is keyed by the walk from the package folder,
///  never by an absolute path,
///  so the line that tolerates its finding reads the same in every checkout.
#[test]
fn layout_a_module_outside_the_package_is_keyed_from_the_package() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "pkg/Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    // The package folder is `pkg`, so `../../outside/x.rs` from its `src` folder leaves it.
    fixture.write(
        "pkg/src/lib.rs",
        "#[path = \"../../outside/x.rs\"] mod x;\n",
    );
    fixture.write("outside/x.rs", "pub struct X;\n");
    fixture.write(
        "pkg/monochromatic-lint.config.jsonc",
        r#"[{ "files": ["src/**/*.rs"], "rules": { "rust/file-order-in-folder": { "severity": "error" } } }]"#,
    );
    let found: Run = run(&fixture.path.join("pkg"), &["--debug", "src"], b"");
    assert_eq!(found.status, 1, "{}", found.stderr);
    // The folder is one step above the package, so both keys write it from the package folder.
    for key in [
        "rust/file-order-in-folder: allow-list key: no-top-doc ../outside",
        "rust/file-order-in-folder: allow-list key: missing-entry ../outside x.rs",
    ] {
        assert!(
            found.stderr.contains(key),
            "{key} missing in {}",
            found.stderr
        );
    }
    assert!(
        !found.stderr.contains("allow-list key: /"),
        "a key never holds an absolute path: {}",
        found.stderr
    );
}

/// Make a `uses` link whose parent folders the fixture already holds.
#[cfg(unix)]
fn uses_link(
    root: &Path,
    entry: &str,
    target: &str,
) {
    let path: PathBuf = root.join(entry);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("link parent");
    }
    std::os::unix::fs::symlink(target, &path).expect("symbolic link");
}

/// A use added inside a loop that an `own-file-loop` line tolerates is a finding:
///  the line holds the loop the finding names,
///  so a use that joins it is new code that no line records,
///  as a use joining a tolerated loop of plain folders is a finding.
#[cfg(unix)]
#[test]
fn layout_a_use_that_joins_a_listed_own_file_loop_is_reported() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(
        CONFIG,
        r#"[{ "files": ["src/**/*.rs"], "rules": { "rust/folder-uses-links": { "severity": "error", "allow": "folder.txt" } } }]"#,
    );
    fixture.write("src/lib.rs", "mod a;\nmod g1;\n");
    fixture.write("src/g1/mod.rs", "pub mod own;\npub mod c1;\npub mod c2;\n");
    // The group's own file uses `a`, and `a` reaches a child folder of the group, so the use closes a loop.
    fixture.write("src/g1/own.rs", "use crate::a::Z;\n");
    fixture.write("src/g1/c1/mod.rs", "pub struct Y;\n");
    fixture.write("src/g1/c2/mod.rs", "pub struct W;\n");
    fixture.write("src/a/mod.rs", "pub mod x;\n");
    fixture.write("src/a/x.rs", "use crate::g1::c1::Y;\npub struct Z;\n");
    uses_link(&fixture.path, "src/a/uses/c1", "../../g1/c1");
    // The list holds today's findings: the group's own file, the use that closes the loop, and the other use inside it.
    fixture.write(
        "folder.txt",
        "group-file src/g1 own.rs\nown-file-loop src/g1 -> src/a\nuses src/a -> src/g1/c1\n",
    );
    let listed: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(
        (listed.status, listed.stdout.as_str()),
        (0, ""),
        "{}",
        listed.stderr
    );
    // New code adds an entry `b` inside that loop: `a` uses `b`, and `b` reaches another child folder of the group.
    fixture.write("src/lib.rs", "mod a;\nmod b;\nmod g1;\n");
    fixture.write(
        "src/a/x.rs",
        "use crate::g1::c1::Y;\nuse crate::b::B;\npub struct Z;\n",
    );
    fixture.write("src/b/mod.rs", "pub mod y;\n");
    fixture.write("src/b/y.rs", "use crate::g1::c2::W;\npub struct B;\n");
    uses_link(&fixture.path, "src/a/uses/b", "../../b");
    uses_link(&fixture.path, "src/b/uses/c2", "../../g1/c2");
    let grown: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(grown.status, 1, "{}", grown.stderr);
    for part in [
        "src/a uses the folder src/b",
        "src/b uses the folder src/g1/c2",
        "a tolerated loop may only lose uses",
    ] {
        assert!(
            grown.stdout.contains(part),
            "{part} missing in {}",
            grown.stdout
        );
    }
    // The two lines still hold their findings: the run reports the enlargement, not a dead line.
    assert!(
        !grown.stdout.contains("tolerates nothing any more"),
        "{}",
        grown.stdout
    );
}

/// A group nested in another group is searched like any other group:
///  a use written in its own file that closes a loop is reported,
///  so new code cannot add one inside the own file of a nested group.
#[cfg(unix)]
#[test]
fn layout_a_nested_groups_own_file_use_that_closes_a_loop_is_reported() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(
        CONFIG,
        r#"[{ "files": ["src/**/*.rs"], "rules": { "rust/folder-uses-links": { "severity": "error", "allow": "folder.txt" } } }]"#,
    );
    fixture.write("src/lib.rs", "mod g;\nmod x;\n");
    fixture.write("src/g/mod.rs", "pub mod c;\npub mod inner;\n");
    fixture.write("src/g/c/mod.rs", "pub struct C;\n");
    fixture.write("src/g/inner/mod.rs", "pub mod d;\npub mod own;\n");
    fixture.write("src/g/inner/d/mod.rs", "pub struct D;\n");
    // The own file of the nested group `g/inner` uses `x`, which reaches nothing of `g` yet.
    fixture.write("src/g/inner/own.rs", "use crate::x::X;\n");
    fixture.write("src/x/mod.rs", "pub mod y;\n");
    fixture.write("src/x/y.rs", "pub struct X;\n");
    fixture.write(
        "folder.txt",
        "nested-group src/g/inner\ngroup-file src/g/inner own.rs\n",
    );
    let listed: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(
        (listed.status, listed.stdout.as_str()),
        (0, ""),
        "{}",
        listed.stderr
    );
    // New code makes `x` use a child folder of `g`, which closes the loop through the nested group's own use.
    fixture.write("src/x/y.rs", "use crate::g::c::C;\npub struct X;\n");
    uses_link(&fixture.path, "src/x/uses/c", "../../g/c");
    let grown: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(grown.status, 1, "{}", grown.stderr);
    for part in [
        "These entries use each other in a loop",
        "src/g/inner uses src/x (`crate::x::X`)",
        "src/x uses src/g (`crate::g::c::C` at src/x/y.rs:1)",
    ] {
        assert!(
            grown.stdout.contains(part),
            "{part} missing in {}",
            grown.stdout
        );
    }
    assert!(
        grown
            .stderr
            .contains("allow-list key: own-file-loop src/g/inner -> src/x"),
        "{}",
        grown.stderr
    );
}

/// The own-file loop's key names the used folder, not the entry it belongs to,
///  so a line records exactly the use the finding reports.
#[test]
fn layout_the_own_file_loop_key_names_the_used_folder() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(
        CONFIG,
        r#"[{ "files": ["src/**/*.rs"], "rules": { "rust/folder-uses-links": { "severity": "error", "allow": "folder.txt" } } }]"#,
    );
    fixture.write("src/lib.rs", "mod g;\nmod h;\n");
    fixture.write("src/g/mod.rs", "pub mod own;\npub mod c1;\n");
    fixture.write("src/g/own.rs", "use crate::h::c1::X;\n");
    fixture.write("src/g/c1/mod.rs", "pub struct Y;\n");
    fixture.write("src/h/mod.rs", "pub mod c1;\n");
    fixture.write("src/h/c1/mod.rs", "use crate::g::c1::Y;\npub struct X;\n");
    // `h/c1` uses `g/c1`, which is inside the group `g`, so its folder links it and `h`'s summary links `g`.
    uses_link(&fixture.path, "src/h/c1/uses/c1", "../../../g/c1");
    uses_link(&fixture.path, "src/h/uses/g", "../../g");
    fixture.write("folder.txt", "");
    let looped: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(looped.status, 1, "{}", looped.stderr);
    // The used folder is `h/c1`, which sits inside the group `h`; the key names the folder the use reaches.
    assert!(
        looped
            .stderr
            .contains("allow-list key: own-file-loop src/g -> src/h/c1"),
        "{}",
        looped.stderr
    );
    fixture.write(
        "folder.txt",
        "own-file-loop src/g -> src/h/c1\ngroup-file src/g own.rs\nuses src/h/c1 -> src/g/c1\n",
    );
    let listed: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(
        (listed.status, listed.stdout.as_str()),
        (0, ""),
        "{}",
        listed.stderr
    );
}

/// A glob that two module contexts resolve to two different folders is reported
///  with one message, naming the folder whose text sorts first,
///  so the report never depends on which `mod` line stands first.
#[test]
fn layout_a_glob_names_the_same_folder_whichever_mod_line_comes_first() {
    fn messages_of(lines: &str) -> Vec<String> {
        let fixture: Fixture = Fixture::new();
        fixture.write(
            "Cargo.toml",
            "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
        );
        fixture.write(
            CONFIG,
            r#"[{ "files": ["src/**/*.rs"], "rules": { "rust/folder-uses-links": { "severity": "error" } } }]"#,
        );
        fixture.write("src/lib.rs", lines);
        fixture.write("src/a/mod.rs", "");
        fixture.write(
            "src/m/mod.rs",
            "#[path = \"../shared.rs\"] mod shared;\nmod a;\n",
        );
        fixture.write("src/m/a/mod.rs", "");
        // From inside the folder, `super` is `m` and its child `a`; from the crate root it is `src` and its child `a`.
        fixture.write("src/shared.rs", "pub use super::{a::*, a::*};\n");
        let found: Run = run(&fixture.path, &["src"], b"");
        assert_eq!(found.status, 1, "{}", found.stderr);
        let mut messages: Vec<String> = records(found.stdout.as_str())
            .iter()
            .filter_map(|record| return record["message"].as_str().map(String::from))
            .filter(|message| return message.contains("re-exports every item"))
            .collect::<Vec<String>>();
        messages.sort();
        messages.dedup();
        return messages;
    }
    let after: Vec<String> = messages_of("mod a;\nmod m;\nmod shared;\n");
    let before: Vec<String> = messages_of("mod shared;\nmod m;\nmod a;\n");
    assert_eq!(after.len(), 1, "{after:#?}");
    assert_eq!(before, after);
    assert!(
        after[0].contains("the module in src/a, so a path"),
        "{after:#?}"
    );
}

/// A folder whose name ends in whitespace is refused before any check,
///  because every key about a file below it would end with that character.
#[test]
fn layout_a_folder_name_that_ends_in_whitespace_is_refused() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write("src/lib.rs", "#[path = \"m /mod.rs\"] mod m;\n");
    fixture.write("src/m /mod.rs", "pub mod y;\n");
    fixture.write("src/m /y.rs", "pub struct Y;\n");
    fixture.write(
        CONFIG,
        r#"[{ "files": ["src/**/*.rs"], "rules": { "rust/folder-uses-links": { "severity": "error" }, "rust/file-order-in-folder": { "severity": "error" } } }]"#,
    );
    let refused: Run = run(&fixture.path, &["src"], b"");
    assert_eq!(refused.status, 2, "{}", refused.stdout);
    let reported: Vec<serde_json::Value> = records(refused.stdout.as_str());
    assert_eq!(reported.len(), 1, "{}", refused.stdout);
    let message: &str = reported[0]["message"].as_str().expect("message");
    assert!(
        message.contains("The layout rules stop at the folder src/m\\u{20},"),
        "{message}"
    );
    assert_eq!(reported[0]["code"], PROCESSING_FAILURE, "{message}");
}

/// Every whitespace character the list reader trims ends a name the refusal reports,
///  not only a space, a tab and a no-break space.
#[test]
fn layout_a_name_that_ends_in_any_other_whitespace_is_refused() {
    for (name, escaped) in [
        ("x.rs\u{2003}", "\\u{2003}"),
        ("x.rs\u{3000}", "\\u{3000}"),
        ("x.rs\u{85}", "\\u{85}"),
        ("x.rs\u{2028}", "\\u{2028}"),
    ] {
        let fixture: Fixture = Fixture::new();
        fixture.write(
            "Cargo.toml",
            "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
        );
        fixture.write(
            "src/lib.rs",
            format!("#[path = \"{name}\"] mod x;\n").as_str(),
        );
        fixture.write(format!("src/{name}").as_str(), "pub struct X;\n");
        fixture.write(
            CONFIG,
            r#"[{ "files": ["src/**/*.rs"], "rules": { "rust/folder-uses-links": { "severity": "error" }, "rust/file-order-in-folder": { "severity": "error" } } }]"#,
        );
        let refused: Run = run(&fixture.path, &["src"], b"");
        assert_eq!(refused.status, 2, "{escaped}: {}", refused.stdout);
        let reported: Vec<serde_json::Value> = records(refused.stdout.as_str());
        assert_eq!(reported.len(), 1, "{escaped}: {}", refused.stdout);
        let message: &str = reported[0]["message"].as_str().expect("message");
        assert!(
            message.contains(format!("The layout rules stop at src/x.rs{escaped},").as_str()),
            "{escaped}: {message}"
        );
        assert_eq!(
            reported[0]["code"], PROCESSING_FAILURE,
            "{escaped}: {message}"
        );
    }
}

/// A name that holds a line break is refused before any check,
///  because a list line ends at a newline, so no key could name the file.
#[test]
fn layout_a_name_that_holds_a_line_break_is_refused() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    // The attribute writes the escape `\\n`, which the module tree reads as the line break in the file's name.
    fixture.write("src/lib.rs", "#[path = \"a\\nb.rs\"] mod x;\n");
    fixture.write("src/a\nb.rs", "pub struct X;\n");
    fixture.write(
        CONFIG,
        r#"[{ "files": ["src/**/*.rs"], "rules": { "rust/folder-uses-links": { "severity": "error" }, "rust/file-order-in-folder": { "severity": "error" } } }]"#,
    );
    let refused: Run = run(&fixture.path, &["src"], b"");
    assert_eq!(refused.status, 2, "{}", refused.stdout);
    let reported: Vec<serde_json::Value> = records(refused.stdout.as_str());
    assert_eq!(reported.len(), 1, "{}", refused.stdout);
    let message: &str = reported[0]["message"].as_str().expect("message");
    for part in [
        "The layout rules stop at src/a\\u{a}b.rs,",
        "holds a line break, which ends a list line",
    ] {
        assert!(message.contains(part), "{part} missing in {message}");
    }
    assert_eq!(reported[0]["code"], PROCESSING_FAILURE, "{message}");
}

/// The walk back is breadth first, so a loop finding spells the fewest uses that lead back,
///  not any path the walk happened to take.
#[test]
fn layout_the_own_file_loop_spells_the_fewest_uses_back() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(
        CONFIG,
        r#"[{ "files": ["src/**/*.rs"], "rules": { "rust/folder-uses-links": { "severity": "error" } } }]"#,
    );
    fixture.write("src/lib.rs", "mod a;\nmod c;\nmod d;\nmod g;\nmod x;\n");
    fixture.write("src/a/mod.rs", "pub mod f;\n");
    fixture.write("src/a/f.rs", "use crate::g::c0::G;\npub struct A;\n");
    fixture.write("src/c/mod.rs", "pub mod f;\n");
    fixture.write("src/c/f.rs", "use crate::d::D;\npub struct C;\n");
    fixture.write("src/d/mod.rs", "pub mod f;\n");
    fixture.write("src/d/f.rs", "use crate::g::c0::G;\npub struct D;\n");
    fixture.write("src/g/mod.rs", "pub mod c0;\npub mod own;\n");
    fixture.write("src/g/c0/mod.rs", "pub struct G;\n");
    fixture.write("src/g/own.rs", "use crate::x::X;\n");
    fixture.write("src/x/mod.rs", "pub mod f;\n");
    // `x` reaches `g` in two uses through `a`, and in three through `c` and `d`.
    fixture.write(
        "src/x/f.rs",
        "use crate::a::A;\nuse crate::c::C;\npub struct X;\n",
    );
    let looped: Run = run(&fixture.path, &["src"], b"");
    assert_eq!(looped.status, 1, "{}", looped.stderr);
    for part in [
        "src/x uses src/a (`crate::a::A` at src/x/f.rs:1)",
        "src/a uses src/g (`crate::g::c0::G` at src/a/f.rs:1)",
    ] {
        assert!(
            looped.stdout.contains(part),
            "{part} missing in {}",
            looped.stdout
        );
    }
    assert!(
        !looped.stdout.contains("src/c uses src/d"),
        "{}",
        looped.stdout
    );
}

/// A top doc entry whose backticks hold no name is keyed without the name,
///  because a key that ended in the space would read as if the space were not there,
///  so no list line could hold the finding.
#[test]
fn layout_a_top_doc_entry_that_names_nothing_is_keyed_without_the_name() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write("src/lib.rs", "mod a;\n");
    fixture.write(
        "src/a/mod.rs",
        "//! Files in this folder,\n//! each using only files listed before it:\n//!\n//! - ``: nothing.\n//! - ` `: a blank name.\nmod one;\n",
    );
    fixture.write("src/a/one.rs", "pub struct One;\n");
    fixture.write(
        CONFIG,
        r#"[{ "files": ["src/**/*.rs"], "rules": { "rust/file-order-in-folder": { "severity": "error", "allow": "file.txt" } } }]"#,
    );
    fixture.write("file.txt", "");
    let first: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(first.status, 1, "{}", first.stderr);
    for key in [
        "allow-list key: listed-twice src/a/mod.rs\n",
        "allow-list key: foreign-name src/a/mod.rs\n",
        "allow-list key: foreign-name#2 src/a/mod.rs\n",
    ] {
        assert!(
            first.stderr.contains(key),
            "{key} missing in {}",
            first.stderr
        );
    }
    // The printed keys written as the list hold every finding of the tree.
    fixture.write(
        "file.txt",
        "listed-twice src/a/mod.rs\nforeign-name src/a/mod.rs\nforeign-name#2 src/a/mod.rs\nmissing-entry src/a/mod.rs one.rs\n",
    );
    let listed: Run = run(&fixture.path, &["src"], b"");
    assert_eq!(
        (listed.status, listed.stdout.as_str()),
        (0, ""),
        "{}",
        listed.stderr
    );
}

/// A configuration selecting the folder rule with a list beside the fixture.
const FOLDER_RULES: &str = r#"[{ "files": ["src/**/*.rs"], "rules": { "rust/folder-uses-links": { "severity": "error", "allow": "folder.txt" } } }]"#;

/// A configuration selecting the file rule with a list beside the fixture.
const FILE_RULES: &str = r#"[{ "files": ["src/**/*.rs"], "rules": { "rust/file-order-in-folder": { "severity": "error", "allow": "file.txt" } } }]"#;

/// Write the package of a group whose own file closes a loop that two routes reach `g` by:
///  one through `a`, one through `c` and `d`.
fn write_two_route_loop(fixture: &Fixture) {
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FOLDER_RULES);
    fixture.write("src/lib.rs", "mod a;\nmod c;\nmod d;\nmod g;\nmod x;\n");
    fixture.write("src/g/mod.rs", "pub mod c0;\npub mod own;\n");
    fixture.write("src/g/c0/mod.rs", "pub struct G;\n");
    fixture.write("src/g/own.rs", "use crate::x::X;\n");
    fixture.write("src/x/mod.rs", "pub mod f;\n");
    fixture.write(
        "src/x/f.rs",
        "use crate::a::A;\nuse crate::c::C;\npub struct X;\n",
    );
    fixture.write("src/a/mod.rs", "pub mod f;\n");
    fixture.write("src/a/f.rs", "use crate::g::c0::G;\npub struct A;\n");
    fixture.write("src/c/mod.rs", "pub mod f;\n");
    fixture.write("src/c/f.rs", "use crate::d::D;\npub struct C;\n");
    fixture.write("src/d/mod.rs", "pub mod f;\n");
    fixture.write("src/d/f.rs", "use crate::g::c0::G;\npub struct D;\n");
    uses_link(&fixture.path, "src/x/uses/a", "../../a");
    uses_link(&fixture.path, "src/x/uses/c", "../../c");
    uses_link(&fixture.path, "src/a/uses/c0", "../../g/c0");
    uses_link(&fixture.path, "src/c/uses/d", "../../d");
    uses_link(&fixture.path, "src/d/uses/c0", "../../g/c0");
}

/// The shapes one `uses` line of the loop above writes.
const TWO_ROUTE_USES: [&str; 5] = [
    "uses src/x -> src/a",
    "uses src/x -> src/c",
    "uses src/a -> src/g/c0",
    "uses src/c -> src/d",
    "uses src/d -> src/g/c0",
];

/// Write the smallest package the release pass made for this defect:
///  a group's own file uses `x`, `x` reaches `g` by two routes,
///  one directly and one through `a`.
fn write_two_route_min(fixture: &Fixture) {
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FOLDER_RULES);
    fixture.write("src/lib.rs", "mod a;\nmod g;\nmod x;\n");
    fixture.write("src/g/mod.rs", "pub mod c0;\npub mod own;\n");
    fixture.write("src/g/c0/mod.rs", "pub struct G;\n");
    fixture.write("src/g/own.rs", "use crate::x::X;\n");
    fixture.write("src/x/mod.rs", "pub mod f;\n");
    fixture.write(
        "src/x/f.rs",
        "use crate::a::A;\nuse crate::g::c0::G;\npub struct X;\n",
    );
    fixture.write("src/a/mod.rs", "pub mod f;\n");
    fixture.write("src/a/f.rs", "use crate::g::c0::G;\npub struct A;\n");
    uses_link(&fixture.path, "src/x/uses/a", "../../a");
    uses_link(&fixture.path, "src/x/uses/c0", "../../g/c0");
    uses_link(&fixture.path, "src/a/uses/c0", "../../g/c0");
}

/// The shapes one `uses` line of the smallest loop writes.
const TWO_ROUTE_MIN_USES: [&str; 3] = [
    "uses src/x -> src/a",
    "uses src/x -> src/g/c0",
    "uses src/a -> src/g/c0",
];

/// A loop a group's own file closes is held by one line per use inside it,
///  so the keys of a tree's own run write the whole loop and a second run has nothing to report.
///  Both the smallest tree the release pass made and the one that makes the longer route visible are run.
#[cfg(unix)]
#[test]
fn layout_the_printed_keys_hold_an_own_file_loop_with_two_routes() {
    let min: Fixture = Fixture::new();
    write_two_route_min(&min);
    let (min_keys, min_held): (Vec<String>, Run) = run_with_its_printed_keys(&min, "folder.txt");
    assert!(
        min_keys
            .iter()
            .any(|key| return key == "own-file-loop src/g -> src/x"),
        "{min_keys:?}"
    );
    for key in TWO_ROUTE_MIN_USES {
        assert!(
            min_keys.iter().any(|printed| return printed == key),
            "{key} missing in {min_keys:?}"
        );
    }
    assert_eq!(
        (min_held.status, min_held.stdout.as_str()),
        (0, ""),
        "{} {}",
        min_held.stdout,
        min_held.stderr
    );
    let fixture: Fixture = Fixture::new();
    write_two_route_loop(&fixture);
    fixture.write("folder.txt", "");
    let (keys, held): (Vec<String>, Run) = run_with_its_printed_keys(&fixture, "folder.txt");
    assert!(
        keys.iter()
            .any(|key| return key == "own-file-loop src/g -> src/x"),
        "{keys:?}"
    );
    for key in TWO_ROUTE_USES {
        assert!(
            keys.iter().any(|printed| return printed == key),
            "{key} missing in {keys:?}"
        );
    }
    assert_eq!(
        (held.status, held.stdout.as_str()),
        (0, ""),
        "{} {}",
        held.stdout,
        held.stderr
    );
}

/// A `uses` line that records one use inside a loop a group's own file closes stays alive,
///  so the loop can be written down and the line is not reported as dead.
#[cfg(unix)]
#[test]
fn layout_a_use_on_the_list_stays_alive_inside_an_own_file_loop() {
    let fixture: Fixture = Fixture::new();
    write_two_route_loop(&fixture);
    let mut list: String = String::from("group-file src/g own.rs\nown-file-loop src/g -> src/x\n");
    for line in TWO_ROUTE_USES {
        list.push_str(format!("{line}\n").as_str());
    }
    fixture.write("folder.txt", list.as_str());
    let listed: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(
        (listed.status, listed.stdout.as_str()),
        (0, ""),
        "{} {}",
        listed.stdout,
        listed.stderr
    );
}

/// A use added between two entries a recorded loop already names is a finding,
///  because the `uses` lines record the folder pairs the loop had and not the entry pair alone.
#[cfg(unix)]
#[test]
fn layout_a_use_added_between_two_entries_a_recorded_loop_names_is_reported() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FOLDER_RULES);
    fixture.write("src/lib.rs", "mod a;\nmod g;\n");
    fixture.write("src/g/mod.rs", "pub mod c1;\npub mod c2;\npub mod own;\n");
    fixture.write("src/g/own.rs", "use crate::a::Z;\n");
    fixture.write("src/g/c1/mod.rs", "pub struct Y;\n");
    fixture.write("src/g/c2/mod.rs", "pub struct W;\n");
    fixture.write("src/a/mod.rs", "pub mod x;\n");
    fixture.write("src/a/x.rs", "use crate::g::c1::Y;\npub struct Z;\n");
    uses_link(&fixture.path, "src/a/uses/c1", "../../g/c1");
    let (_, listed): (Vec<String>, Run) = run_with_its_printed_keys(&fixture, "folder.txt");
    assert_eq!(
        (listed.status, listed.stdout.as_str()),
        (0, ""),
        "{} {}",
        listed.stdout,
        listed.stderr
    );
    // New code adds a second use between the same two entries, from a folder pair no line records.
    fixture.write("src/a/mod.rs", "pub mod x;\npub mod z;\n");
    fixture.write("src/a/z.rs", "use crate::g::c2::W;\n");
    uses_link(&fixture.path, "src/a/uses/c2", "../../g/c2");
    let grown: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(grown.status, 1, "{} {}", grown.stdout, grown.stderr);
    assert!(
        grown.stdout.contains("src/a uses the folder src/g/c2"),
        "{}",
        grown.stdout
    );
    assert!(
        grown.stdout.contains("a tolerated loop may only lose uses"),
        "{}",
        grown.stdout
    );
    assert!(
        !grown.stdout.contains("tolerates nothing any more"),
        "{}",
        grown.stdout
    );
}

/// A dense loop that many groups close through one folder is written down in one round:
///  every use inside it is printed as the line that records it, and the second run is clean.
#[cfg(unix)]
#[test]
fn layout_the_printed_keys_hold_a_dense_own_file_loop() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FOLDER_RULES);
    let entries: usize = 4;
    let mut lib: String = String::new();
    for index in 0..entries {
        lib.push_str(format!("mod f{index};\nmod g{index};\n").as_str());
    }
    fixture.write("src/lib.rs", lib.as_str());
    for index in 0..entries {
        // What: one group whose own file uses `f0`, and one child folder it holds.
        // Why: every group's own use enters `f0`, and `f0` uses every group's child, so all of them make one loop.
        fixture.write(
            format!("src/g{index}/mod.rs").as_str(),
            format!("pub mod c{index};\npub mod own;\n").as_str(),
        );
        fixture.write(
            format!("src/g{index}/own.rs").as_str(),
            "use crate::f0::X;\n",
        );
        fixture.write(
            format!("src/g{index}/c{index}/mod.rs").as_str(),
            "pub struct X;\n",
        );
        // What: one plain folder that uses every group's child and every later plain folder.
        // Why: those uses are inside the one loop, so the round trip has to record every one of them.
        let mut body: String = String::new();
        for group in 0..entries {
            body.push_str(format!("use crate::g{group}::c{group}::X;\n").as_str());
        }
        for later in index + 1..entries {
            body.push_str(format!("use crate::f{later}::X;\n").as_str());
        }
        body.push_str("pub struct X;\n");
        fixture.write(format!("src/f{index}/mod.rs").as_str(), body.as_str());
        for group in 0..entries {
            let link: String = format!("src/f{index}/uses/c{group}");
            let target: String = format!("../../g{group}/c{group}");
            uses_link(&fixture.path, link.as_str(), target.as_str());
        }
        for later in index + 1..entries {
            let link: String = format!("src/f{index}/uses/f{later}");
            let target: String = format!("../../f{later}");
            uses_link(&fixture.path, link.as_str(), target.as_str());
        }
    }
    fixture.write("folder.txt", "");
    let (keys, held): (Vec<String>, Run) = run_with_its_printed_keys(&fixture, "folder.txt");
    let loops: usize = keys
        .iter()
        .filter(|key| return key.starts_with("own-file-loop "))
        .count();
    assert_eq!(loops, entries, "{keys:?}");
    assert!(
        keys.iter()
            .any(|key| return key == "uses src/f0 -> src/g0/c0"),
        "{keys:?}"
    );
    assert_eq!(
        (held.status, held.stdout.as_str()),
        (0, ""),
        "{} {}",
        held.stdout,
        held.stderr
    );
}

/// Many generated packages whose loops a group's own file closes are written down in one round each,
///  and the control proves the run reaches some of them.
#[cfg(unix)]
#[test]
fn layout_the_printed_keys_hold_random_own_file_trees() {
    let mut with_a_loop: usize = 0;
    for seed in 0..12 {
        let fixture: Fixture = Fixture::new();
        fixture.write(
            "Cargo.toml",
            "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
        );
        fixture.write(CONFIG, FOLDER_RULES);
        fixture.write("folder.txt", "");
        write_generated_tree(&fixture, seed);
        let (keys, held): (Vec<String>, Run) = run_with_its_printed_keys(&fixture, "folder.txt");
        if keys
            .iter()
            .any(|key| return key.starts_with("own-file-loop "))
        {
            with_a_loop += 1;
        }
        assert_eq!(
            (held.status, held.stdout.as_str()),
            (0, ""),
            "seed {seed}: keys {keys:?}, stdout {}, stderr {}",
            held.stdout,
            held.stderr
        );
    }
    assert!(
        with_a_loop > 0,
        "no generated package held a loop, so the run proves nothing"
    );
}

/// The ABI string of an `extern` block is keyed with every run of whitespace squeezed to one space,
///  so a key never holds a line break and a list line can record the finding.
#[test]
fn layout_an_extern_block_abi_string_holding_a_line_break_is_keyed_on_one_line() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FILE_RULES);
    fixture.write("file.txt", "");
    fixture.write("src/lib.rs", "mod m;\n");
    fixture.write(
        "src/m/mod.rs",
        "//! Files in this folder,\n//! each using only files listed before it:\n//!\n//! - `one.rs`: one.\nmod one;\nextern \"C\nx\" { }\n",
    );
    fixture.write("src/m/one.rs", "pub struct One;\n");
    let (keys, held): (Vec<String>, Run) = run_with_its_printed_keys(&fixture, "file.txt");
    assert!(
        keys.iter()
            .any(|key| return key == "index-item src/m/mod.rs extern-block-\"C x\""),
        "{keys:?}"
    );
    for key in &keys {
        assert!(!key.contains('\n'), "{key:?} holds a line break");
    }
    assert_eq!(
        (held.status, held.stdout.as_str()),
        (0, ""),
        "{} {}",
        held.stdout,
        held.stderr
    );
}

/// A `uses` entry whose name holds a line break is refused before any check,
///  because no list line could hold a key that names it.
#[cfg(unix)]
#[test]
fn layout_a_uses_entry_name_that_holds_a_line_break_is_refused() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FOLDER_RULES);
    fixture.write("folder.txt", "");
    fixture.write("src/lib.rs", "mod a;\nmod b;\n");
    fixture.write("src/a/mod.rs", "pub mod x;\n");
    fixture.write("src/a/x.rs", "use crate::b::B;\npub struct X;\n");
    fixture.write("src/b/mod.rs", "pub struct B;\n");
    uses_link(&fixture.path, "src/a/uses/b\nc", "../../b");
    let refused: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(refused.status, 2, "{} {}", refused.stdout, refused.stderr);
    let reported: Vec<serde_json::Value> = records(refused.stdout.as_str());
    assert_eq!(reported.len(), 1, "{}", refused.stdout);
    let message: &str = reported[0]["message"].as_str().expect("message");
    for part in [
        "The layout rules stop at src/a/uses/b\\u{a}c,",
        "holds a line break, which ends a list line",
    ] {
        assert!(message.contains(part), "{part} missing in {message}");
    }
    assert_eq!(reported[0]["code"], PROCESSING_FAILURE, "{message}");
}

/// A use of an entry the loop does not reach is not a use that joins the loop,
///  so a recorded loop reports nothing about it.
#[cfg(unix)]
#[test]
fn layout_a_use_of_an_entry_outside_a_recorded_own_file_loop_is_not_reported() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FOLDER_RULES);
    fixture.write("src/lib.rs", "mod a;\nmod g1;\nmod z;\n");
    fixture.write("src/g1/mod.rs", "pub mod c1;\npub mod c2;\npub mod own;\n");
    fixture.write("src/g1/own.rs", "use crate::a::Z;\n");
    fixture.write("src/g1/c1/mod.rs", "pub struct Y;\n");
    fixture.write("src/g1/c2/mod.rs", "pub struct W;\n");
    fixture.write("src/z/mod.rs", "pub struct Q;\n");
    fixture.write("src/a/mod.rs", "pub mod x;\n");
    // `a` closes the loop through `g1/c1` and also uses `z`, which sits outside every loop.
    fixture.write(
        "src/a/x.rs",
        "use crate::g1::c1::Y;\nuse crate::z::Q;\npub struct Z;\n",
    );
    uses_link(&fixture.path, "src/a/uses/c1", "../../g1/c1");
    uses_link(&fixture.path, "src/a/uses/z", "../../z");
    fixture.write("folder.txt", "");
    let (_, held): (Vec<String>, Run) = run_with_its_printed_keys(&fixture, "folder.txt");
    assert_eq!(
        (held.status, held.stdout.as_str()),
        (0, ""),
        "{} {}",
        held.stdout,
        held.stderr
    );
}

/// A use that a loop of plain folders reports is not reported again by the check for a loop
///  a group's own file closes.
#[cfg(unix)]
#[test]
fn layout_a_use_a_folder_loop_reports_is_not_reported_again() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FOLDER_RULES);
    fixture.write("src/lib.rs", "mod a;\nmod b;\nmod g1;\n");
    fixture.write("src/g1/mod.rs", "pub mod c1;\npub mod c2;\npub mod own;\n");
    fixture.write("src/g1/own.rs", "use crate::a::Z;\n");
    fixture.write("src/g1/c1/mod.rs", "pub struct Y;\n");
    fixture.write("src/g1/c2/mod.rs", "pub struct W;\n");
    fixture.write("src/a/mod.rs", "pub mod x;\n");
    // `a` and `b` use each other, so the folder loop reports that use itself.
    fixture.write(
        "src/a/x.rs",
        "use crate::g1::c1::Y;\nuse crate::b::B;\npub struct Z;\n",
    );
    fixture.write("src/b/mod.rs", "use crate::a::Z;\npub struct B;\n");
    uses_link(&fixture.path, "src/a/uses/b", "../../b");
    uses_link(&fixture.path, "src/a/uses/c1", "../../g1/c1");
    uses_link(&fixture.path, "src/b/uses/a", "../../a");
    fixture.write(
        "folder.txt",
        "group-file src/g1 own.rs\nown-file-loop src/g1 -> src/a\nuses src/a -> src/g1/c1\n",
    );
    let listed: Run = run(&fixture.path, &["--debug", "src"], b"");
    let reported: Vec<serde_json::Value> = records(listed.stdout.as_str());
    let joining: usize = reported
        .iter()
        .filter(|record| {
            return record["message"]
                .as_str()
                .expect("message")
                .contains("a tolerated loop may only lose uses");
        })
        .count();
    assert_eq!(joining, 0, "{}", listed.stdout);
    let loops: usize = reported
        .iter()
        .filter(|record| {
            return record["message"]
                .as_str()
                .expect("message")
                .contains("These folders use each other in a loop");
        })
        .count();
    assert_eq!(loops, 1, "{}", listed.stdout);
}

/// A use between two folders of one entry makes no edge of the entry graph,
///  so the check for a loop a group's own file closes reports nothing about it.
#[cfg(unix)]
#[test]
fn layout_a_use_between_two_folders_of_one_entry_is_not_reported() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FOLDER_RULES);
    fixture.write("src/lib.rs", "mod a;\nmod g1;\n");
    fixture.write("src/g1/mod.rs", "pub mod c1;\npub mod c2;\npub mod own;\n");
    fixture.write("src/g1/own.rs", "use crate::a::Z;\n");
    fixture.write("src/g1/c1/mod.rs", "pub struct Y;\n");
    // One child of the group uses another, which joins no two entries.
    fixture.write("src/g1/c2/mod.rs", "use crate::g1::c1::Y;\npub struct W;\n");
    fixture.write("src/a/mod.rs", "pub mod x;\n");
    fixture.write("src/a/x.rs", "use crate::g1::c1::Y;\npub struct Z;\n");
    uses_link(&fixture.path, "src/a/uses/c1", "../../g1/c1");
    uses_link(&fixture.path, "src/g1/c2/uses/c1", "../../c1");
    fixture.write("folder.txt", "");
    let (_, held): (Vec<String>, Run) = run_with_its_printed_keys(&fixture, "folder.txt");
    assert_eq!(
        (held.status, held.stdout.as_str()),
        (0, ""),
        "{} {}",
        held.stdout,
        held.stderr
    );
}

/// A loop one of whose own-file findings the list does not record is still reported,
///  and the uses inside it are printed as the lines that record them.
#[cfg(unix)]
#[test]
fn layout_a_loop_with_one_own_file_finding_off_the_list_keys_its_uses() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FOLDER_RULES);
    fixture.write("src/lib.rs", "mod e;\nmod g1;\nmod g2;\n");
    fixture.write("src/g1/mod.rs", "pub mod c1;\npub mod own;\n");
    fixture.write("src/g1/c1/mod.rs", "pub struct Y;\n");
    fixture.write("src/g2/mod.rs", "pub mod c2;\npub mod own;\n");
    fixture.write("src/g2/c2/mod.rs", "use crate::e::E;\npub struct W;\n");
    fixture.write("src/e/mod.rs", "use crate::g1::c1::Y;\npub struct E;\n");
    // Each group's own file uses the other's child, so one loop holds both own-file findings.
    fixture.write("src/g1/own.rs", "use crate::g2::c2::W;\n");
    fixture.write("src/g2/own.rs", "use crate::g1::c1::Y;\n");
    uses_link(&fixture.path, "src/e/uses/c1", "../../g1/c1");
    uses_link(&fixture.path, "src/g2/c2/uses/e", "../../e");
    uses_link(&fixture.path, "src/g2/uses/e", "../../e");
    let (keys, _): (Vec<String>, Run) = run_with_its_printed_keys(&fixture, "folder.txt");
    let loops: usize = keys
        .iter()
        .filter(|key| return key.starts_with("own-file-loop "))
        .count();
    assert_eq!(loops, 2, "{keys:?}");
    // The list records one of the two own-file findings, so the loop is still reported,
    // and the uses inside it are printed as the lines that record them.
    fixture.write(
        "folder.txt",
        "group-file src/g1 own.rs\ngroup-file src/g2 own.rs\nown-file-loop src/g2 -> src/g1/c1\n",
    );
    let partial: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(partial.status, 1, "{}", partial.stderr);
    let partial_keys: Vec<String> = printed_keys(partial.stderr.as_str());
    assert!(
        partial_keys
            .iter()
            .any(|key| return key == "uses src/e -> src/g1/c1"),
        "{partial_keys:?}"
    );
    assert!(
        partial
            .stdout
            .contains("These entries use each other in a loop"),
        "{}",
        partial.stdout
    );
    // A reported loop holds no use of its own: the uses inside it are the lines that record it,
    // so none of them is a use that joins a loop the list tolerates.
    let joining: usize = records(partial.stdout.as_str())
        .iter()
        .filter(|record| {
            return record["message"]
                .as_str()
                .expect("message")
                .contains("a tolerated loop may only lose uses");
        })
        .count();
    assert_eq!(joining, 0, "{}", partial.stdout);
}

/// A loop that two groups' own files close reports one finding for each of them,
///  and the walk of the outer group's own file steps over the nested group's own use.
#[cfg(unix)]
#[test]
fn layout_a_loop_through_two_groups_own_files_reports_each_own_file() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FOLDER_RULES);
    fixture.write("folder.txt", "");
    fixture.write("src/lib.rs", "mod g;\nmod h;\n");
    fixture.write("src/g/mod.rs", "pub mod c;\npub mod inner;\n");
    fixture.write("src/g/c/mod.rs", "pub struct C;\n");
    fixture.write("src/g/inner/mod.rs", "pub mod d;\npub mod own;\n");
    fixture.write("src/g/inner/d/mod.rs", "pub struct D;\n");
    fixture.write("src/g/inner/own.rs", "use crate::h::c2::W;\n");
    fixture.write("src/h/mod.rs", "pub mod c2;\npub mod own;\n");
    fixture.write("src/h/c2/mod.rs", "pub struct W;\n");
    fixture.write("src/h/own.rs", "use crate::g::c::C;\n");
    let (keys, _): (Vec<String>, Run) = run_with_its_printed_keys(&fixture, "folder.txt");
    for key in [
        "own-file-loop src/g/inner -> src/h/c2",
        "own-file-loop src/h -> src/g/c",
    ] {
        assert!(
            keys.iter().any(|printed| return printed == key),
            "{key} missing in {keys:?}"
        );
    }
}

/// A folder whose name holds a line break is refused before any check,
///  because a key about a file inside it would hold that name.
#[cfg(unix)]
#[test]
fn layout_a_folder_name_that_holds_a_line_break_is_refused() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FILE_RULES);
    fixture.write("file.txt", "");
    fixture.write("src/lib.rs", "#[path = \"m\nn/mod.rs\"] mod m;\n");
    fixture.write(
        "src/m\nn/mod.rs",
        "//! Files in this folder,\n//! each using only files listed before it:\n//!\n//! - `y.rs`: one.\nmod y;\n",
    );
    fixture.write("src/m\nn/y.rs", "pub struct Y;\n");
    let refused: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(refused.status, 2, "{} {}", refused.stdout, refused.stderr);
    let reported: Vec<serde_json::Value> = records(refused.stdout.as_str());
    assert_eq!(reported.len(), 1, "{}", refused.stdout);
    let message: &str = reported[0]["message"].as_str().expect("message");
    assert!(
        message.contains("The layout rules stop at the folder src/m\\u{a}n,"),
        "{message}"
    );
    assert_eq!(reported[0]["code"], PROCESSING_FAILURE, "{message}");
}

/// A folder whose name holds a space still records a loop its group's own file closes:
///  every key the rule prints for such a folder is a line the reader reads back.
#[cfg(unix)]
#[test]
fn layout_the_printed_keys_hold_a_loop_through_a_folder_name_with_a_space() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FOLDER_RULES);
    fixture.write(
        "src/lib.rs",
        "#[path = \"a b/mod.rs\"]\nmod a;\nmod g;\nmod x;\n",
    );
    fixture.write("src/g/mod.rs", "pub mod c0;\npub mod own;\n");
    fixture.write("src/g/c0/mod.rs", "pub struct G;\n");
    fixture.write("src/g/own.rs", "use crate::x::X;\n");
    fixture.write("src/x/mod.rs", "pub mod f;\n");
    fixture.write(
        "src/x/f.rs",
        "use crate::a::A;\nuse crate::g::c0::G;\npub struct X;\n",
    );
    fixture.write("src/a b/mod.rs", "pub mod f;\n");
    fixture.write("src/a b/f.rs", "use crate::g::c0::G;\npub struct A;\n");
    uses_link(&fixture.path, "src/x/uses/a b", "../../a b");
    uses_link(&fixture.path, "src/x/uses/c0", "../../g/c0");
    uses_link(&fixture.path, "src/a b/uses/c0", "../../g/c0");
    let (keys, held): (Vec<String>, Run) = run_with_its_printed_keys(&fixture, "folder.txt");
    for key in [
        "own-file-loop src/g -> src/x",
        "uses src/a b -> src/g/c0",
        "uses src/x -> src/a b",
        "uses src/x -> src/g/c0",
    ] {
        assert!(
            keys.iter().any(|printed| return printed == key),
            "{key} missing in {keys:?}"
        );
    }
    assert_eq!(
        (held.status, held.stdout.as_str()),
        (0, ""),
        "{} {}",
        held.stdout,
        held.stderr
    );
}

/// A module a `#[path]` names outside the package is a folder of the same tree,
///  so the keys the rule prints about it are lines the reader reads back.
#[cfg(unix)]
#[test]
fn layout_the_printed_keys_hold_a_loop_through_a_module_outside_the_package() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "pkg/Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(format!("pkg/{CONFIG}").as_str(), FOLDER_RULES);
    fixture.write(
        "pkg/src/lib.rs",
        "#[path = \"../../outside/mod.rs\"]\nmod ext;\nmod g;\n",
    );
    fixture.write("pkg/src/g/mod.rs", "pub mod c0;\npub mod own;\n");
    fixture.write("pkg/src/g/c0/mod.rs", "pub struct G;\n");
    fixture.write("pkg/src/g/own.rs", "use crate::ext::E;\n");
    fixture.write("outside/mod.rs", "use crate::g::c0::G;\npub struct E;\n");
    let package: std::path::PathBuf = fixture.path.join("pkg");
    fixture.write("pkg/folder.txt", "");
    let first: Run = run(&package, &["--debug", "src"], b"");
    let keys: Vec<String> = printed_keys(first.stderr.as_str());
    for key in [
        "missing-link ../outside -> src/g/c0",
        "own-file-loop src/g -> ../outside",
        "uses ../outside -> src/g/c0",
    ] {
        assert!(
            keys.iter().any(|printed| return printed == key),
            "{key} missing in {keys:?}"
        );
    }
    let mut text: String = keys.join("\n");
    text.push('\n');
    fixture.write("pkg/folder.txt", text.as_str());
    let held: Run = run(&package, &["--debug", "src"], b"");
    assert_eq!(
        (held.status, held.stdout.as_str()),
        (0, ""),
        "{} {}",
        held.stdout,
        held.stderr
    );
}

/// A folder whose name holds a backslash is refused before any check,
///  because every key writes that name as another folder.
#[cfg(unix)]
#[test]
fn layout_a_folder_name_that_holds_a_backslash_is_refused() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FOLDER_RULES);
    fixture.write("folder.txt", "");
    fixture.write("src/lib.rs", "#[path = \"a\\\\b/mod.rs\"]\nmod a;\n");
    fixture.write("src/a\\b/mod.rs", "pub struct A;\n");
    let refused: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(refused.status, 2, "{} {}", refused.stdout, refused.stderr);
    let reported: Vec<serde_json::Value> = records(refused.stdout.as_str());
    assert_eq!(reported.len(), 1, "{}", refused.stdout);
    let message: &str = reported[0]["message"].as_str().expect("message");
    for part in [
        "The layout rules stop at the folder src/a\\b,",
        "holds a backslash",
    ] {
        assert!(message.contains(part), "{part} missing in {message}");
    }
    assert_eq!(reported[0]["code"], PROCESSING_FAILURE, "{message}");
}

/// A folder whose name holds the words a `uses` line sets its arrow off with is refused before any check,
///  because that line could not say where the used folder starts.
#[cfg(unix)]
#[test]
fn layout_a_folder_name_that_holds_the_arrow_words_is_refused() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FOLDER_RULES);
    fixture.write("folder.txt", "");
    fixture.write("src/lib.rs", "#[path = \"a -> b/mod.rs\"]\nmod a;\n");
    fixture.write("src/a -> b/mod.rs", "pub struct A;\n");
    let refused: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(refused.status, 2, "{} {}", refused.stdout, refused.stderr);
    let reported: Vec<serde_json::Value> = records(refused.stdout.as_str());
    assert_eq!(reported.len(), 1, "{}", refused.stdout);
    let message: &str = reported[0]["message"].as_str().expect("message");
    for part in [
        "The layout rules stop at the folder src/a -> b,",
        "its name holds ` -> `,",
        "would read as another pair of folders",
    ] {
        assert!(message.contains(part), "{part} missing in {message}");
    }
    assert_eq!(reported[0]["code"], PROCESSING_FAILURE, "{message}");
}

/// A folder name that would be misread on a `uses` line is refused only while the folder rule is selected:
///  the file rule reads no `uses` line,
///  so it reads such a tree as alpha.17 did rather than stop the run.
#[cfg(unix)]
#[test]
fn layout_the_file_rule_alone_reads_a_folder_named_the_arrow() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FILE_RULES);
    fixture.write("file.txt", "");
    fixture.write("src/lib.rs", "#[path = \"->/mod.rs\"]\nmod a;\n");
    fixture.write(
        "src/->/mod.rs",
        "//! Files in this folder,\n//! each using only files listed before it:\n//!\n//! - `one.rs`: one.\nmod one;\n",
    );
    fixture.write("src/->/one.rs", "pub struct One;\n");
    let read: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(
        (read.status, read.stdout.as_str()),
        (0, ""),
        "{} {}",
        read.stdout,
        read.stderr
    );
}

/// Write a package of two folders that use each other, with the list of one shape of `uses` line:
///  the loop itself is recorded, so a run reports only what the line handed in changes.
#[cfg(unix)]
fn write_two_folder_loop(
    fixture: &Fixture,
    list: &str,
) {
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FOLDER_RULES);
    fixture.write("src/lib.rs", "mod a;\nmod b;\n");
    fixture.write("src/a/mod.rs", "use crate::b::B;\npub struct A;\n");
    fixture.write("src/b/mod.rs", "use crate::a::A;\npub struct B;\n");
    fixture.write("folder.txt", list);
}

/// A folder whose name ends in the arrow is refused before any check,
///  because the `uses` line that would name it reads as another pair of folders:
///  `uses src/a -> -> src/g/c0` reads as the folder `src/a` using `-> src/g/c0`.
#[cfg(unix)]
#[test]
fn layout_a_folder_name_that_ends_in_the_arrow_is_refused() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FOLDER_RULES);
    fixture.write("folder.txt", "");
    fixture.write(
        "src/lib.rs",
        "#[path = \"a ->/mod.rs\"]\nmod a;\nmod g;\nmod x;\n",
    );
    fixture.write("src/g/mod.rs", "pub mod c0;\npub mod own;\n");
    fixture.write("src/g/c0/mod.rs", "pub struct G;\n");
    fixture.write("src/g/own.rs", "use crate::x::X;\n");
    fixture.write("src/x/mod.rs", "pub mod f;\n");
    fixture.write(
        "src/x/f.rs",
        "use crate::a::A;\nuse crate::g::c0::G;\npub struct X;\n",
    );
    fixture.write("src/a ->/mod.rs", "pub mod f;\n");
    fixture.write("src/a ->/f.rs", "use crate::g::c0::G;\npub struct A;\n");
    let refused: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(refused.status, 2, "{} {}", refused.stdout, refused.stderr);
    let reported: Vec<serde_json::Value> = records(refused.stdout.as_str());
    assert_eq!(reported.len(), 1, "{}", refused.stdout);
    let message: &str = reported[0]["message"].as_str().expect("message");
    for part in [
        "The layout rules stop at the folder src/a ->,",
        "its name ends with ` ->`,",
        "would read as another pair of folders",
    ] {
        assert!(message.contains(part), "{part} missing in {message}");
    }
    assert_eq!(reported[0]["code"], PROCESSING_FAILURE, "{message}");
}

/// A folder whose name begins with the arrow is refused before any check,
///  because a `uses` line naming it would be read from an arrow inside its own name.
#[cfg(unix)]
#[test]
fn layout_a_folder_name_that_begins_with_the_arrow_is_refused() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FOLDER_RULES);
    fixture.write("folder.txt", "");
    fixture.write("src/lib.rs", "#[path = \"-> a/mod.rs\"]\nmod a;\n");
    fixture.write("src/-> a/mod.rs", "pub struct A;\n");
    let refused: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(refused.status, 2, "{} {}", refused.stdout, refused.stderr);
    let reported: Vec<serde_json::Value> = records(refused.stdout.as_str());
    assert_eq!(reported.len(), 1, "{}", refused.stdout);
    let message: &str = reported[0]["message"].as_str().expect("message");
    for part in [
        "The layout rules stop at the folder src/-> a,",
        "its name begins with `-> `,",
        "would read as another pair of folders",
    ] {
        assert!(message.contains(part), "{part} missing in {message}");
    }
    assert_eq!(reported[0]["code"], PROCESSING_FAILURE, "{message}");
}

/// A folder named exactly the arrow is refused before any check,
///  because the space a `uses` line sets its words off with closes another arrow with its name.
#[cfg(unix)]
#[test]
fn layout_a_folder_name_that_is_the_arrow_is_refused() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FOLDER_RULES);
    fixture.write("folder.txt", "");
    fixture.write("src/lib.rs", "#[path = \"->/mod.rs\"]\nmod a;\n");
    fixture.write("src/->/mod.rs", "pub struct A;\n");
    let refused: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(refused.status, 2, "{} {}", refused.stdout, refused.stderr);
    let reported: Vec<serde_json::Value> = records(refused.stdout.as_str());
    assert_eq!(reported.len(), 1, "{}", refused.stdout);
    let message: &str = reported[0]["message"].as_str().expect("message");
    for part in [
        "The layout rules stop at the folder src/->,",
        "its name is `->`,",
        "would read as another pair of folders",
    ] {
        assert!(message.contains(part), "{part} missing in {message}");
    }
    assert_eq!(reported[0]["code"], PROCESSING_FAILURE, "{message}");
}

/// A folder ending in the arrow beside one beginning with it is refused whichever comes first,
///  because one printed line, `uses src/a -> -> -> src/b`, could name two pairs at once.
#[cfg(unix)]
#[test]
fn layout_two_folder_names_that_could_share_one_uses_line_are_refused() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FOLDER_RULES);
    fixture.write("folder.txt", "");
    fixture.write(
        "src/lib.rs",
        "#[path = \"a ->/mod.rs\"]\nmod a;\n#[path = \"-> b/mod.rs\"]\nmod b;\n",
    );
    fixture.write("src/a ->/mod.rs", "pub struct A;\n");
    fixture.write("src/-> b/mod.rs", "pub struct B;\n");
    let refused: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(refused.status, 2, "{} {}", refused.stdout, refused.stderr);
    let reported: Vec<serde_json::Value> = records(refused.stdout.as_str());
    assert_eq!(reported.len(), 1, "{}", refused.stdout);
    let message: &str = reported[0]["message"].as_str().expect("message");
    assert!(
        message.contains("The layout rules stop at the folder src/a ->,")
            || message.contains("The layout rules stop at the folder src/-> b,"),
        "{message}"
    );
    assert!(
        message.contains("would read as another pair of folders"),
        "{message}"
    );
    assert_eq!(reported[0]["code"], PROCESSING_FAILURE, "{message}");
}

/// A `uses` line whose spacing leaves the name before the arrow ending in whitespace names a folder the package lacks,
///  because no folder name of a tree the rules accept ends in whitespace:
///  the run reports that one line dead, naming the folder it read,
///  and goes on reading the rest of the list rather than hiding every other finding of the rule.
#[cfg(unix)]
#[test]
fn layout_a_use_line_whose_name_before_the_arrow_ends_in_whitespace_is_reported_dead() {
    let fixture: Fixture = Fixture::new();
    write_two_folder_loop(
        &fixture,
        "uses src/a  -> src/b\nuses src/b -> src/a\nmissing-link src/a -> src/b\nmissing-link src/b -> src/a\n",
    );
    let listed: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(listed.status, 1, "{} {}", listed.stdout, listed.stderr);
    let reported: Vec<serde_json::Value> = records(listed.stdout.as_str());
    // What: the finding about the misspaced line, and the use its dead line leaves unrecorded inside the loop the rest of the list tolerates.
    // Why: both are what the whole list being read costs, where alpha.18 stopped the rule and showed neither.
    assert_eq!(reported.len(), 2, "{}", listed.stdout);
    let dead: &serde_json::Value = reported
        .iter()
        .find(|item| return item["filename"] == "folder.txt")
        .expect("the dead line is reported on its own line");
    let message: &str = dead["message"].as_str().expect("message");
    for part in [
        "The allow-list entry `uses src/a  -> src/b`",
        "names the folder `src/a `",
        "No source folder of this package has that name",
    ] {
        assert!(message.contains(part), "{part} missing in {message}");
    }
    assert_eq!(dead["code"], "rust/folder-uses-links", "{message}");
    // The control: the loop the other three lines tolerate is not reported,
    //  so the reader of a column-aligned list keeps every other answer of the rule.
    assert!(
        listed
            .stdout
            .contains("one loop the allow list tolerates today"),
        "the rest of the list is read: {}",
        listed.stdout
    );
    assert!(
        !listed.stdout.contains("use each other in a loop"),
        "the loop stays tolerated: {}",
        listed.stdout
    );
}

/// A `uses` line with two spaces in front of a name reads a folder whose name starts with a space,
///  and the finding for the dead entry names the folder it read and says the package holds none of it,
///  so a hand-aligned line is re-spaced rather than deleted.
#[cfg(unix)]
#[test]
fn layout_a_use_line_with_a_doubled_space_in_front_of_a_name_names_the_folder_it_read() {
    let fixture: Fixture = Fixture::new();
    write_two_folder_loop(
        &fixture,
        "uses  src/a -> src/b\nuses src/b -> src/a\nmissing-link src/a -> src/b\nmissing-link src/b -> src/a\n",
    );
    let listed: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(listed.status, 1, "{} {}", listed.stdout, listed.stderr);
    for part in [
        "The allow-list entry `uses  src/a -> src/b`",
        "names the folder ` src/a`",
        "No source folder of this package has that name",
    ] {
        assert!(
            listed.stdout.contains(part),
            "{part} missing in {}",
            listed.stdout
        );
    }
}

/// A `uses` line with a word after the folder it names reads that word as part of the name,
///  and the finding for the dead entry says no folder of the package is called that,
///  so a mistyped line is named as mistyped rather than only told to be deleted.
#[cfg(unix)]
#[test]
fn layout_a_use_line_with_a_word_after_its_folder_names_the_folder_the_tree_lacks() {
    let fixture: Fixture = Fixture::new();
    write_two_folder_loop(
        &fixture,
        "uses src/a -> src/b extra\nuses src/b -> src/a\nmissing-link src/a -> src/b\nmissing-link src/b -> src/a\n",
    );
    let listed: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(listed.status, 1, "{} {}", listed.stdout, listed.stderr);
    for part in [
        "The allow-list entry `uses src/a -> src/b extra`",
        "names the folder `src/b extra`",
        "No source folder of this package has that name",
    ] {
        assert!(
            listed.stdout.contains(part),
            "{part} missing in {}",
            listed.stdout
        );
    }
}

/// An entry of a `uses` folder is no path,
///  so a name that holds a backslash keeps it in the key about the entry,
///  which the next run reads back as the same entry.
#[cfg(unix)]
#[test]
fn layout_a_uses_entry_name_that_holds_a_backslash_keeps_its_name_in_its_key() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FOLDER_RULES);
    fixture.write("src/lib.rs", "mod a;\nmod b;\n");
    fixture.write("src/a/mod.rs", "pub mod x;\n");
    fixture.write("src/a/x.rs", "use crate::b::B;\npub struct X;\n");
    fixture.write("src/b/mod.rs", "pub struct B;\n");
    uses_link(&fixture.path, "src/a/uses/b\\c", "../../b");
    let (keys, held): (Vec<String>, Run) = run_with_its_printed_keys(&fixture, "folder.txt");
    assert!(
        keys.iter()
            .any(|key| return key == "link src/a/uses/b\\c named-after-another-folder"),
        "{keys:?}"
    );
    assert_eq!(
        (held.status, held.stdout.as_str()),
        (0, ""),
        "{keys:?}\n{}\n{}",
        held.stdout,
        held.stderr
    );
}

/// An entry of a `uses` folder never appears on a `uses` line,
///  so a name that holds the arrow words is no ambiguity and its key holds the name as written.
#[cfg(unix)]
#[test]
fn layout_a_uses_entry_name_that_holds_the_arrow_words_keeps_its_name_in_its_key() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FOLDER_RULES);
    fixture.write("src/lib.rs", "mod a;\nmod b;\n");
    fixture.write("src/a/mod.rs", "pub mod x;\n");
    fixture.write("src/a/x.rs", "use crate::b::B;\npub struct X;\n");
    fixture.write("src/b/mod.rs", "pub struct B;\n");
    uses_link(&fixture.path, "src/a/uses/b -> c", "../../b");
    let (keys, held): (Vec<String>, Run) = run_with_its_printed_keys(&fixture, "folder.txt");
    assert!(
        keys.iter()
            .any(|key| return key == "link src/a/uses/b -> c named-after-another-folder"),
        "{keys:?}"
    );
    assert_eq!(
        (held.status, held.stdout.as_str()),
        (0, ""),
        "{keys:?}\n{}\n{}",
        held.stdout,
        held.stderr
    );
}

/// A file name is no path, so a key about a file whose name holds a backslash keeps that name,
///  and the key the rule prints is the line the next run reads back.
#[test]
fn layout_a_glob_key_about_a_file_name_that_holds_a_backslash_keeps_its_name() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FOLDER_RULES);
    fixture.write("src/lib.rs", "mod m;\n#[path = \"a\\\\b.rs\"]\nmod q;\n");
    fixture.write("src/m.rs", "pub struct M;\n");
    fixture.write("src/a\\b.rs", "pub use crate::m::*;\n");
    let (keys, held): (Vec<String>, Run) = run_with_its_printed_keys(&fixture, "folder.txt");
    assert!(
        keys.iter()
            .any(|key| return key == "glob src/a\\b.rs crate::m"),
        "{keys:?}"
    );
    assert_eq!(
        (held.status, held.stdout.as_str()),
        (0, ""),
        "{keys:?}\n{}\n{}",
        held.stdout,
        held.stderr
    );
}

/// A `file-uses` line is five words set off with one whitespace character each,
///  so a folder whose name holds a space makes six words and the whole list is refused.
///  The refusal names that folder and the space in its name, and what to do about it,
///  because the line it was given reads as the shape the refusal would otherwise only spell out.
///  This is the limit the README's "Known limits" sentence states for the file rule.
#[cfg(unix)]
#[test]
fn layout_a_file_use_line_whose_folder_name_holds_a_space_is_refused() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FILE_RULES);
    fixture.write("src/lib.rs", "#[path = \"a b/mod.rs\"]\nmod m;\n");
    fixture.write(
        "src/a b/mod.rs",
        "//! Files in this folder,\n//! each using only files listed before it:\n//!\n//! - `one.rs`: one.\n//! - `two.rs`: two.\nmod one;\nmod two;\n",
    );
    fixture.write("src/a b/one.rs", "use super::two::T;\npub struct O;\n");
    fixture.write("src/a b/two.rs", "use super::one::O;\npub struct T;\n");
    fixture.write("file.txt", "file-uses src/a b one.rs -> two.rs\n");
    let refused: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(refused.status, 2, "{} {}", refused.stdout, refused.stderr);
    let reported: Vec<serde_json::Value> = records(refused.stdout.as_str());
    assert_eq!(reported.len(), 1, "{}", refused.stdout);
    let message: &str = reported[0]["message"].as_str().expect("message");
    for part in [
        "Line 1",
        "the folder `src/a b`",
        "holds a space",
        "Rename the folder",
    ] {
        assert!(message.contains(part), "{part} missing in {message}");
    }
    assert_eq!(reported[0]["code"], PROCESSING_FAILURE, "{message}");
}

/// A `uses` line between two folders of two different loops records nothing,
///  because a line stays alive only while its two folders sit in one loop together.
#[cfg(unix)]
#[test]
fn layout_a_use_line_joining_two_loops_is_reported_dead() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FOLDER_RULES);
    fixture.write("src/lib.rs", "mod a;\nmod b;\nmod p;\nmod q;\n");
    fixture.write("src/a/mod.rs", "pub mod x;\n");
    // `a` uses `b`, which uses `a`, and also uses `p`, which uses `q`, which uses `p`.
    // `p` never reaches `a`, so the two folders are in two loops and not in one.
    fixture.write(
        "src/a/x.rs",
        "use crate::b::B;\nuse crate::p::P;\npub struct A;\n",
    );
    fixture.write("src/b/mod.rs", "use crate::a::A;\npub struct B;\n");
    fixture.write("src/p/mod.rs", "use crate::q::Q;\npub struct P;\n");
    fixture.write("src/q/mod.rs", "use crate::p::P;\npub struct Q;\n");
    uses_link(&fixture.path, "src/a/uses/b", "../../b");
    uses_link(&fixture.path, "src/a/uses/p", "../../p");
    uses_link(&fixture.path, "src/b/uses/a", "../../a");
    uses_link(&fixture.path, "src/p/uses/q", "../../q");
    uses_link(&fixture.path, "src/q/uses/p", "../../p");
    fixture.write("folder.txt", "uses src/a -> src/p\n");
    let listed: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(listed.status, 1, "{} {}", listed.stdout, listed.stderr);
    assert!(
        listed
            .stdout
            .contains("The allow-list entry `uses src/a -> src/p`"),
        "{}",
        listed.stdout
    );
    assert!(
        listed.stdout.contains("tolerates nothing any more"),
        "{}",
        listed.stdout
    );
}

/// An entry of a `uses` folder whose name ends in a space is refused before any check,
///  because a list line is read without the spaces around it.
#[cfg(unix)]
#[test]
fn layout_a_uses_entry_name_that_ends_in_a_space_is_refused() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FOLDER_RULES);
    fixture.write("folder.txt", "");
    fixture.write("src/lib.rs", "mod a;\nmod b;\n");
    fixture.write("src/a/mod.rs", "pub mod x;\n");
    fixture.write("src/a/x.rs", "use crate::b::B;\npub struct X;\n");
    fixture.write("src/b/mod.rs", "pub struct B;\n");
    uses_link(&fixture.path, "src/a/uses/b ", "../../b");
    let refused: Run = run(&fixture.path, &["--debug", "src"], b"");
    assert_eq!(refused.status, 2, "{} {}", refused.stdout, refused.stderr);
    let reported: Vec<serde_json::Value> = records(refused.stdout.as_str());
    assert_eq!(reported.len(), 1, "{}", refused.stdout);
    let message: &str = reported[0]["message"].as_str().expect("message");
    for part in [
        "The layout rules stop at src/a/uses/b\\u{20},",
        "its name ends in whitespace that an allow-list reader trims away",
    ] {
        assert!(message.contains(part), "{part} missing in {message}");
    }
    assert_eq!(reported[0]["code"], PROCESSING_FAILURE, "{message}");
}

/// An `extern` block's ABI string is keyed with every run of whitespace squeezed to one space,
///  so a key never holds a tab, a no-break space or another character a list line reads away.
#[test]
fn layout_an_extern_block_abi_string_holding_a_tab_is_keyed_with_one_space() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FILE_RULES);
    fixture.write("file.txt", "");
    fixture.write("src/lib.rs", "mod m;\n");
    fixture.write(
        "src/m/mod.rs",
        "//! Files in this folder,\n//! each using only files listed before it:\n//!\n//! - `one.rs`: one.\nmod one;\nextern \"C\tx\" { }\n",
    );
    fixture.write("src/m/one.rs", "pub struct One;\n");
    let (keys, held): (Vec<String>, Run) = run_with_its_printed_keys(&fixture, "file.txt");
    assert!(
        keys.iter()
            .any(|key| return key == "index-item src/m/mod.rs extern-block-\"C x\""),
        "{keys:?}"
    );
    assert_eq!(
        (held.status, held.stdout.as_str()),
        (0, ""),
        "{} {}",
        held.stdout,
        held.stderr
    );
}

/// A folder whose name starts with a space still records a loop its group's own file closes:
///  the key the rule prints puts that one space after the word `uses`, and the reader reads the space back as part of the name.
#[cfg(unix)]
#[test]
fn layout_the_printed_keys_hold_a_loop_through_a_folder_whose_name_starts_with_a_space() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FOLDER_RULES);
    fixture.write(
        "src/lib.rs",
        "#[path = \"../ a/mod.rs\"]\nmod a;\nmod g;\nmod x;\n",
    );
    fixture.write("src/g/mod.rs", "pub mod c0;\npub mod own;\n");
    fixture.write("src/g/c0/mod.rs", "pub struct G;\n");
    fixture.write("src/g/own.rs", "use crate::x::X;\n");
    fixture.write("src/x/mod.rs", "pub mod f;\n");
    fixture.write(
        "src/x/f.rs",
        "use crate::a::A;\nuse crate::g::c0::G;\npub struct X;\n",
    );
    fixture.write(" a/mod.rs", "pub mod f;\n");
    fixture.write(" a/f.rs", "use crate::g::c0::G;\npub struct A;\n");
    let (keys, held): (Vec<String>, Run) = run_with_its_printed_keys(&fixture, "folder.txt");
    for key in [
        "uses  a -> src/g/c0",
        "uses src/x ->  a",
        "uses src/x -> src/g/c0",
    ] {
        assert!(
            keys.iter().any(|printed| return printed == key),
            "{key} missing in {keys:?}"
        );
    }
    assert_eq!(
        (held.status, held.stdout.as_str()),
        (0, ""),
        "{} {}",
        held.stdout,
        held.stderr
    );
}

/// The allow-list keys one `--debug` run printed, in the order it printed them.
fn printed_keys(text: &str) -> Vec<String> {
    let mut keys: Vec<String> = Vec::<String>::new();
    for line in text.lines() {
        if let Some((_, key)) = line.split_once("allow-list key: ") {
            keys.push(key.to_string());
        }
    }
    return keys;
}

/// Write the keys a `--debug` run printed as the fixture's list, then run again.
/// Why:
///  This is what a round does with a rule it switches on,
///  and a tree's own findings must survive it.
///  The list starts empty, so the first run reports every finding of the tree.
fn run_with_its_printed_keys(
    fixture: &Fixture,
    list: &str,
) -> (Vec<String>, Run) {
    fixture.write(list, "");
    let first: Run = run(&fixture.path, &["--debug", "src"], b"");
    let keys: Vec<String> = printed_keys(first.stderr.as_str());
    let mut text: String = keys.join("\n");
    if !text.is_empty() {
        text.push('\n');
    }
    fixture.write(list, text.as_str());
    let second: Run = run(&fixture.path, &["--debug", "src"], b"");
    return (keys, second);
}

/// A top doc that spells the marker nearly reports a finding of its own,
///  and the keys the run prints are the lines the list reads back.
#[test]
fn layout_the_printed_keys_tolerate_a_near_miss_marker() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
    );
    fixture.write(CONFIG, FILE_RULES);
    fixture.write(
        "src/lib.rs",
        "//! Files in this folder,\n//! each using only the files listed before it:\n//! - `one.rs`: one.\nmod one;\n",
    );
    fixture.write("src/one.rs", "");
    fixture.write("file.txt", "");
    let reported: Run = run(&fixture.path, &["src"], b"");
    assert_eq!(reported.status, 1, "{}", reported.stderr);
    for part in [
        "nearly spells the file-list marker at line 1",
        "Files in this folder, each using only the files listed before it:",
    ] {
        assert!(
            reported.stdout.contains(part),
            "{part} missing in {}",
            reported.stdout
        );
    }
    // The keys the run printed are the list's lines, and the next run reports nothing.
    let (keys, held): (Vec<String>, Run) = run_with_its_printed_keys(&fixture, "file.txt");
    assert!(
        keys.iter()
            .any(|key| return key == "near-miss-marker src/lib.rs"),
        "{keys:?}"
    );
    assert_eq!(
        (held.status, held.stdout.as_str()),
        (0, ""),
        "{} {}",
        held.stdout,
        held.stderr
    );
}

/// A deterministic stand-in for a random generator,
///  so a control over many generated packages builds the same packages on every run.
struct Rolls {
    /// The generator's whole state.
    state: u64,
}

impl Rolls {
    /// Start one sequence at a seed.
    fn new(seed: u64) -> Rolls {
        return Rolls {
            state: seed
                .wrapping_mul(2_863_933_555_777_941_757)
                .wrapping_add(3_037_000_493),
        };
    }

    /// The next value.
    fn next(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        return self.state >> 33;
    }

    /// A value below one bound.
    fn below(
        &mut self,
        bound: usize,
    ) -> usize {
        return (self.next() % bound as u64) as usize;
    }

    /// Whether a chance of one in a hundred came up.
    fn chance(
        &mut self,
        percent: u64,
    ) -> bool {
        return self.next() % 100 < percent;
    }
}

/// Write one generated package:
///  one-folder entries and groups whose own files may use any entry,
///  while every other use points at an entry later in one random order,
///  so a loop can only arise through a group's own file.
///  No `uses` link is written,
///  so the round trip of its printed keys records the missing links too.
fn write_generated_tree(
    fixture: &Fixture,
    seed: u64,
) {
    let mut rolls: Rolls = Rolls::new(seed);
    let plain: usize = 2 + rolls.below(4);
    let groups: usize = 1 + rolls.below(2);
    let mut children: Vec<usize> = Vec::<usize>::new();
    for _ in 0..groups {
        children.push(1 + rolls.below(2));
    }
    // What: every entry, in a random order.
    // Why: a use written outside a group's own file may only point at a later entry,
    // so a loop can only be closed by a group's own file.
    let mut order: Vec<String> = Vec::<String>::new();
    for index in 0..plain {
        order.push(format!("p{index}"));
    }
    for index in 0..groups {
        order.push(format!("g{index}"));
    }
    for index in (1..order.len()).rev() {
        let other: usize = rolls.below(index + 1);
        order.swap(index, other);
    }
    // What: the position of each entry in that order.
    // Why: "later" is the only direction a use outside a group's own file may take.
    let mut rank: Vec<(String, usize)> = Vec::<(String, usize)>::new();
    for (position, name) in order.iter().enumerate() {
        rank.push((name.clone(), position));
    }
    let position_of = |name: &str| -> usize {
        for (known, position) in &rank {
            if known == name {
                return *position;
            }
        }
        return usize::MAX;
    };
    let mut lib: String = String::new();
    for name in &order {
        lib.push_str(format!("mod {name};\n").as_str());
    }
    fixture.write("src/lib.rs", lib.as_str());
    for name in &order {
        // What: one entry's module file, with the uses it is allowed to write.
        // Why: `own` is a group's own file, which may use any entry, and `forward_only` is every other file.
        let mut body: String = String::from("//! One entry.\n");
        for other in &order {
            if other == name {
                continue;
            }
            if position_of(other) <= position_of(name) {
                continue;
            }
            if other.starts_with('p') {
                if rolls.chance(35) {
                    body.push_str(format!("use crate::{other}::X;\n").as_str());
                }
                continue;
            }
            let index: usize = other[1..].parse::<usize>().expect("numbered group");
            let child: usize = rolls.below(children[index]);
            if rolls.chance(35) {
                body.push_str(format!("use crate::{other}::c{child}::X;\n").as_str());
            }
        }
        if name.starts_with('p') {
            fixture.write(format!("src/{name}/mod.rs").as_str(), body.as_str());
            continue;
        }
        let index: usize = name[1..].parse::<usize>().expect("numbered group");
        let mut index_file: String = String::from("//! One group.\npub mod own;\n");
        for child in 0..children[index] {
            index_file.push_str(format!("pub mod c{child};\n").as_str());
            fixture.write(
                format!("src/{name}/c{child}/mod.rs").as_str(),
                "//! One child.\npub struct X;\n",
            );
        }
        fixture.write(format!("src/{name}/mod.rs").as_str(), index_file.as_str());
        // What: the group's own file, which may use any entry, so it may close a loop.
        // Why: the rule reports a use in a group's own file that closes one, and it is what the round trip records.
        let mut own: String = String::from("//! The group's own file.\n");
        for other in &order {
            if other == name {
                continue;
            }
            let target: String = if other.starts_with('p') {
                format!("crate::{other}::X")
            } else {
                let other_index: usize = other[1..].parse::<usize>().expect("numbered group");
                let child: usize = rolls.below(children[other_index]);
                format!("crate::{other}::c{child}::X")
            };
            if rolls.chance(40) {
                own.push_str(format!("use {target};\n").as_str());
            }
        }
        fixture.write(format!("src/{name}/own.rs").as_str(), own.as_str());
        // What: the same entry's child folders, which may only point forward.
        // Why: their uses can never close a loop, so only a group's own file can.
        for child in 0..children[index] {
            let mut child_body: String = String::from("//! One child.\n");
            for other in &order {
                if position_of(other) <= position_of(name) {
                    continue;
                }
                let target: String = if other.starts_with('p') {
                    format!("crate::{other}::X")
                } else {
                    let other_index: usize = other[1..].parse::<usize>().expect("numbered group");
                    let chosen: usize = rolls.below(children[other_index]);
                    format!("crate::{other}::c{chosen}::X")
                };
                if rolls.chance(30) {
                    child_body.push_str(format!("use {target};\n").as_str());
                }
            }
            child_body.push_str("pub struct X;\n");
            fixture.write(
                format!("src/{name}/c{child}/mod.rs").as_str(),
                child_body.as_str(),
            );
        }
    }
}

/// The unit-test scratch root,
/// shared with the library test build and placed in the build directory.
#[path = "test_scratch.rs"]
mod test_scratch;
